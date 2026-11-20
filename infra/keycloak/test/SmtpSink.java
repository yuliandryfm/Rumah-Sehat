// Penangkap SMTP khusus test realm Keycloak (F1-T31). BUKAN relay email (relay: F1-T36, ADR-026).
//
// Dijalankan oleh scripts/keycloak-check.sh di kontainer image Keycloak yang sama (JDK 21 + jdk.compiler,
// mode berkas sumber tunggal `java SmtpSink.java`), di namespace jaringan kontainer Keycloak, sehingga
// hanya terjangkau dari 127.0.0.1 kontainer itu. Tanpa TLS dan tanpa AUTH: hanya untuk data uji sintetis.
// Setiap pesan ditulis ke <dir>/<n>.eml; test membacanya lewat `docker exec`.
//
// Pakai: java SmtpSink.java <port> <dir>

import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.io.OutputStream;
import java.net.InetAddress;
import java.net.ServerSocket;
import java.net.Socket;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.concurrent.atomic.AtomicInteger;

public class SmtpSink {
    private static final AtomicInteger COUNTER = new AtomicInteger();

    public static void main(String[] args) throws Exception {
        int port = Integer.parseInt(args[0]);
        Path dir = Path.of(args[1]);
        Files.createDirectories(dir);
        try (ServerSocket server = new ServerSocket(port, 16, InetAddress.getLoopbackAddress())) {
            System.out.println("smtp-sink siap di 127.0.0.1:" + port);
            while (true) {
                Socket s = server.accept();
                Thread t = new Thread(() -> handle(s, dir));
                t.setDaemon(true);
                t.start();
            }
        }
    }

    private static void reply(OutputStream out, String line) throws Exception {
        out.write((line + "\r\n").getBytes(StandardCharsets.US_ASCII));
        out.flush();
    }

    private static void handle(Socket s, Path dir) {
        try (s;
             BufferedReader in = new BufferedReader(new InputStreamReader(s.getInputStream(), StandardCharsets.UTF_8));
             OutputStream out = s.getOutputStream()) {
            s.setSoTimeout(30000);
            reply(out, "220 smtp-sink");
            String line;
            while ((line = in.readLine()) != null) {
                String cmd = line.length() >= 4 ? line.substring(0, 4).toUpperCase() : line.toUpperCase();
                switch (cmd) {
                    case "EHLO", "HELO" -> reply(out, "250 smtp-sink");
                    case "MAIL", "RCPT", "RSET", "NOOP" -> reply(out, "250 OK");
                    case "DATA" -> {
                        reply(out, "354 lanjut");
                        StringBuilder msg = new StringBuilder();
                        String d;
                        while ((d = in.readLine()) != null && !d.equals(".")) {
                            msg.append(d.startsWith("..") ? d.substring(1) : d).append("\n");
                        }
                        Path f = dir.resolve(COUNTER.incrementAndGet() + ".eml");
                        Files.writeString(f, msg.toString(), StandardCharsets.UTF_8);
                        reply(out, "250 diterima");
                    }
                    case "QUIT" -> {
                        reply(out, "221 selesai");
                        return;
                    }
                    default -> reply(out, "502 tidak didukung");
                }
            }
        } catch (Exception e) {
            System.out.println("smtp-sink: koneksi berakhir: " + e.getClass().getSimpleName());
        }
    }
}
