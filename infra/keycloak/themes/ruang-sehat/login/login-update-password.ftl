<#-- Turunan login-update-password.ftl keycloak.v2 26.8.2 (TD-27, US-103 AC4d, KEPUTUSAN-198). Bawaan keycloak.v2
     menampilkan kotak "logout-sessions" TIDAK tercentang (password-commons.ftl), sehingga atur ulang password tidak
     mengakhiri sesi lain kecuali pengguna mencentangnya. UpdatePassword 26.8.2 mengakhiri semua sesi lain (online dan
     offline) bila formulir mengirim logout-sessions=on (SRC-770).
     - Bukan aksi dari aplikasi (atur ulang lewat tautan email, aktivasi undangan): logout-sessions=on selalu dikirim
       sebagai isian tersembunyi, tanpa pilihan, dengan kalimat pemberitahuan.
     - Aksi dari aplikasi (kc_action, ubah password saat login, US-103 AC6): kotak pilihan tampil dan tercentang
       bawaan (KEPUTUSAN-198 butir 2). -->
<#import "template.ftl" as layout>
<#import "field.ftl" as field>
<#import "buttons.ftl" as buttons>
<#import "password-validation.ftl" as validator>
<@layout.registrationLayout displayMessage=!messagesPerField.existsError('password','password-confirm'); section>
<!-- template: login-update-password.ftl -->
    <#if section = "header">
        ${msg("updatePasswordTitle")}
    <#elseif section = "form">
        <form id="kc-passwd-update-form" class="${properties.kcFormClass!}" onsubmit="login.disabled = true; return true;" action="${url.loginAction}" method="post" novalidate="novalidate">
            <@field.password name="password-new" label=msg("passwordNew") fieldName="password" autocomplete="new-password" autofocus=true />
            <@field.password name="password-confirm" label=msg("passwordConfirm") autocomplete="new-password" />

            <div class="${properties.kcFormGroupClass!}">
                <#if isAppInitiatedAction??>
                    <div id="kc-form-options" class="${properties.kcFormOptionsClass!}">
                        <div class="${properties.kcFormOptionsWrapperClass!}">
                            <@field.checkbox name="logout-sessions" label=msg("logoutOtherSessions") value=true />
                        </div>
                    </div>
                <#else>
                    <input type="hidden" id="rs-logout-sessions" name="logout-sessions" value="on">
                    <p id="rs-logout-info" class="${properties.kcFormHelperTextClass!}">${msg("rsSemuaSesiDiakhiri")}</p>
                </#if>
            </div>

            <@buttons.actionGroup horizontal=true>
                <#if isAppInitiatedAction??>
                    <@buttons.button id="kc-submit" name="login" label="doSubmit"/>
                    <@buttons.button id="kc-cancel" label="doCancel" name="cancel-aia" type="secondary"/>
                <#else>
                    <@buttons.button id="kc-submit" name="login" label="doSubmit"/>
                </#if>
            </@buttons.actionGroup>
        </form>

        <@validator.templates/>
        <@validator.script field="password-new"/>
    </#if>
</@layout.registrationLayout>
