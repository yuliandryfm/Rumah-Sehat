// Tiruan Amazon SES API v2 khusus test (F1-T36, ADR-026). BUKAN layanan production; tidak pernah memanggil AWS.
//
// Dijalankan oleh scripts/keycloak-check.sh (mode ephemeral) di kontainer image Keycloak yang sama (JDK 21, mode
// berkas sumber tunggal `java SesTiruan.java`), di namespace jaringan kontainer Keycloak, sehingga hanya terjangkau
// dari 127.0.0.1 kontainer itu. HTTPS dengan sertifikat dari CA uji yang dibangkitkan saat test.
//
// Untuk `POST /v2/email/outbound-emails`:
// - memvalidasi tanda tangan SigV4 secara independen dari relay (layanan "ses", region ap-southeast-3, kunci
//   akses dan rahasia contoh dari variabel lingkungan); tanda tangan salah dijawab 403 dan dihitung di
//   <dir-json>/bad-signature;
// - menyimpan isi JSON permintaan ke <dir-json>/<n>.json dan MIME hasil dekode `Content.Raw.Data` ke
//   <dir-mail>/<n>.eml (dibaca test lewat `docker exec`);
// - menjawab sesuai berkas mode /tmp/ses-mode (isi: kode status HTTP; bawaan 200 dengan MessageId).
//
// Pakai: java SesTiruan.java <port> <berkas-p12> <dir-mail> <dir-json>
// Variabel lingkungan: SES_TIRUAN_P12_PASSWORD, SES_TIRUAN_KEY_ID, SES_TIRUAN_SECRET.

import com.sun.net.httpserver.HttpExchange;
import com.sun.net.httpserver.HttpsConfigurator;
import com.sun.net.httpserver.HttpsServer;
import java.io.FileInputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.InetAddress;
import java.net.InetSocketAddress;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.KeyStore;
import java.security.MessageDigest;
import java.util.Base64;
import java.util.HexFormat;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import javax.crypto.Mac;
import javax.crypto.spec.SecretKeySpec;
import javax.net.ssl.KeyManagerFactory;
import javax.net.ssl.SSLContext;

public class SesTiruan {
    private static final AtomicInteger COUNTER = new AtomicInteger();
    private static final Pattern AUTH = Pattern.compile(
        "^AWS4-HMAC-SHA256 Credential=([^/]+)/(\\d{8})/([^/]+)/([^/]+)/aws4_request, SignedHeaders=([a-z0-9;-]+), Signature=([0-9a-f]{64})$");
    private static final Pattern DATA = Pattern.compile("\"Data\"\\s*:\\s*\"([A-Za-z0-9+/=]*)\"");

    public static void main(String[] args) throws Exception {
        int port = Integer.parseInt(args[0]);
        Path mailDir = Path.of(args[2]);
        Path jsonDir = Path.of(args[3]);
        Files.createDirectories(mailDir);
        Files.createDirectories(jsonDir);
        char[] pass = System.getenv("SES_TIRUAN_P12_PASSWORD").toCharArray();
        String keyId = System.getenv("SES_TIRUAN_KEY_ID");
        String secret = System.getenv("SES_TIRUAN_SECRET");

        KeyStore ks = KeyStore.getInstance("PKCS12");
        try (InputStream in = new FileInputStream(args[1])) {
            ks.load(in, pass);
        }
        KeyManagerFactory kmf = KeyManagerFactory.getInstance(KeyManagerFactory.getDefaultAlgorithm());
        kmf.init(ks, pass);
        SSLContext ssl = SSLContext.getInstance("TLS");
        ssl.init(kmf.getKeyManagers(), null, null);

        HttpsServer server = HttpsServer.create(new InetSocketAddress(InetAddress.getLoopbackAddress(), port), 16);
        server.setHttpsConfigurator(new HttpsConfigurator(ssl));
        server.createContext("/", ex -> handle(ex, keyId, secret, mailDir, jsonDir));
        server.start();
        System.out.println("ses-tiruan siap di 127.0.0.1:" + port);
    }

    private static void respond(HttpExchange ex, int status, String body) throws java.io.IOException {
        byte[] b = body.getBytes(StandardCharsets.UTF_8);
        ex.getResponseHeaders().set("Content-Type", "application/json");
        ex.sendResponseHeaders(status, b.length);
        try (OutputStream out = ex.getResponseBody()) {
            out.write(b);
        }
    }

    private static byte[] hmac(byte[] key, String data) throws Exception {
        Mac mac = Mac.getInstance("HmacSHA256");
        mac.init(new SecretKeySpec(key, "HmacSHA256"));
        return mac.doFinal(data.getBytes(StandardCharsets.UTF_8));
    }

    private static String sha256Hex(byte[] data) throws Exception {
        return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(data));
    }

    private static String header(HttpExchange ex, String name) {
        List<String> v = ex.getRequestHeaders().get(name);
        return v == null || v.size() != 1 ? null : v.get(0).trim();
    }

    static boolean signatureValid(HttpExchange ex, byte[] body, String keyId, String secret) throws Exception {
        String auth = header(ex, "Authorization");
        String date = header(ex, "X-Amz-Date");
        if (auth == null || date == null) return false;
        Matcher m = AUTH.matcher(auth);
        if (!m.matches()) return false;
        String day = m.group(2);
        if (!m.group(1).equals(keyId) || !m.group(3).equals("ap-southeast-3") || !m.group(4).equals("ses")
                || !date.startsWith(day)) {
            return false;
        }
        String signed = m.group(5);
        List<String> names = List.of(signed.split(";"));
        if (!names.containsAll(List.of("content-type", "host", "x-amz-date"))) return false;
        StringBuilder canon = new StringBuilder();
        for (String n : names) {
            String v = header(ex, n);
            if (v == null) return false;
            canon.append(n).append(':').append(v).append('\n');
        }
        String creq = ex.getRequestMethod() + "\n" + ex.getRequestURI().getRawPath() + "\n\n" + canon + "\n" + signed
            + "\n" + sha256Hex(body);
        String scope = day + "/ap-southeast-3/ses/aws4_request";
        String sts = "AWS4-HMAC-SHA256\n" + date + "\n" + scope + "\n" + sha256Hex(creq.getBytes(StandardCharsets.UTF_8));
        byte[] k = hmac(("AWS4" + secret).getBytes(StandardCharsets.UTF_8), day);
        k = hmac(k, "ap-southeast-3");
        k = hmac(k, "ses");
        k = hmac(k, "aws4_request");
        String expected = HexFormat.of().formatHex(hmac(k, sts));
        return MessageDigest.isEqual(expected.getBytes(StandardCharsets.US_ASCII),
            m.group(6).getBytes(StandardCharsets.US_ASCII));
    }

    private static void handle(HttpExchange ex, String keyId, String secret, Path mailDir, Path jsonDir) {
        try {
            byte[] body = ex.getRequestBody().readAllBytes();
            if (!ex.getRequestMethod().equals("POST") || !ex.getRequestURI().getRawPath().equals("/v2/email/outbound-emails")) {
                respond(ex, 404, "{}");
                return;
            }
            if (!signatureValid(ex, body, keyId, secret)) {
                Files.writeString(jsonDir.resolve("bad-signature"), "x\n", StandardCharsets.UTF_8,
                    java.nio.file.StandardOpenOption.CREATE, java.nio.file.StandardOpenOption.APPEND);
                respond(ex, 403, "{\"message\":\"signature\"}");
                return;
            }
            Path mode = Path.of("/tmp/ses-mode");
            int status = Files.exists(mode) ? Integer.parseInt(Files.readString(mode).trim()) : 200;
            if (status != 200) {
                // Isi galat sengaja menggemakan permintaan: relay tidak boleh meneruskannya ke Keycloak.
                respond(ex, status, "{\"message\":\"tiruan menolak " + new String(body, StandardCharsets.UTF_8).length() + "\"}");
                return;
            }
            String json = new String(body, StandardCharsets.UTF_8);
            Matcher d = DATA.matcher(json);
            if (!d.find()) {
                respond(ex, 400, "{\"message\":\"Raw.Data\"}");
                return;
            }
            int n = COUNTER.incrementAndGet();
            Files.writeString(jsonDir.resolve(n + ".json"), json, StandardCharsets.UTF_8);
            Files.write(mailDir.resolve(n + ".eml"), Base64.getDecoder().decode(d.group(1)));
            respond(ex, 200, "{\"MessageId\":\"tiruan-" + n + "\"}");
        } catch (Exception e) {
            System.out.println("ses-tiruan: galat " + e.getClass().getSimpleName());
        } finally {
            ex.close();
        }
    }
}
