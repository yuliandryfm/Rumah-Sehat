<#import "template.ftl" as layout>
<#import "field.ftl" as field>
<#import "buttons.ftl" as buttons>
<#import "social-providers.ftl" as identityProviders>
<#import "passkeys.ftl" as passkeys>
<#-- Turunan login.ftl keycloak.v2 26.8.2 dengan satu perubahan: semua kegagalan login (password salah, akun tidak
     ada, terkunci, nonaktif) memakai SATU blok galat yang strukturnya identik (UP-73 opsi B, US-103 AC2, TD-16).
     Tema bawaan menaruh galat password salah di bawah isian dan galat akun nonaktif sebagai banner, sehingga
     struktur halaman membedakan status akun. Di sini displayMessage dimatikan dan galat digambar sendiri. -->
<@layout.registrationLayout displayMessage=false displayInfo=realm.password && realm.registrationAllowed && !registrationDisabled??; section>
<!-- template: login.ftl (ruang-sehat) -->

    <#if section = "header">
        ${msg("loginAccountTitle")}
    <#elseif section = "form">
        <#assign rsGagal = messagesPerField.existsError('username','password') || (message?has_content && message.type == 'error')>
        <div id="kc-login-feedback">
            <#if rsGagal>
                <div id="kc-login-error" class="${properties.kcAlertClass!} pf-m-danger" role="alert">
                    <div class="${properties.kcAlertIconClass!}"><span class="${properties.kcFeedbackErrorIcon!}" aria-hidden="true"></span></div>
                    <span class="${properties.kcAlertTitleClass!} kc-feedback-text">${msg("rsLoginGagal")}</span>
                </div>
            <#elseif message?has_content>
                <div id="kc-login-notice" class="${properties.kcAlertClass!} pf-m-${message.type}" role="status">
                    <div class="${properties.kcAlertIconClass!}"><span class="${properties.kcFeedbackInfoIcon!}" aria-hidden="true"></span></div>
                    <span class="${properties.kcAlertTitleClass!} kc-notice-text">${message.summary}</span>
                </div>
            </#if>
        </div>
        <div id="kc-form">
          <div id="kc-form-wrapper">
            <#if realm.password>
                <form id="kc-form-login" class="${properties.kcFormClass!}" onsubmit="login.disabled = true; return true;" action="${url.loginAction}" method="post" novalidate="novalidate">
                    <#if !usernameHidden??>
                        <#assign label>
                            <#if !realm.loginWithEmailAllowed>${msg("username")}<#elseif !realm.registrationEmailAsUsername>${msg("usernameOrEmail")}<#else>${msg("email")}</#if>
                        </#assign>
                        <@field.input name="username" label=label error=""
                            autofocus=true autocomplete="${(enableWebAuthnConditionalUI?has_content)?then('username webauthn', 'username')}" value=login.username!'' />
                        <@field.password name="password" label=msg("password") error="" forgotPassword=realm.resetPasswordAllowed autofocus=usernameHidden?? autocomplete="current-password">
                            <#if realm.rememberMe && !usernameHidden??>
                                <@field.checkbox name="rememberMe" label=msg("rememberMe") value=login.rememberMe?? />
                            </#if>
                        </@field.password>
                    <#else>
                        <@field.password name="password" label=msg("password") error="" forgotPassword=realm.resetPasswordAllowed autofocus=usernameHidden?? autocomplete="current-password">
                            <#if realm.rememberMe && !usernameHidden??>
                                <@field.checkbox name="rememberMe" label=msg("rememberMe") value=login.rememberMe?? />
                            </#if>
                        </@field.password>
                    </#if>

                    <input type="hidden" id="id-hidden-input" name="credentialId" <#if auth.selectedCredential?has_content>value="${auth.selectedCredential}"</#if>/>
                    <@buttons.loginButton />
                </form>
            </#if>
            </div>
        </div>
        <@passkeys.conditionalUIData />
    <#elseif section = "socialProviders" >
        <#if realm.password && social.providers?? && social.providers?has_content>
            <@identityProviders.show social=social/>
        </#if>
    <#elseif section = "info" >
        <#if realm.password && realm.registrationAllowed && !registrationDisabled??>
            <div id="kc-registration-container">
                <div id="kc-registration">
                    <span>${msg("noAccount")} <a href="${url.registrationUrl}">${msg("doRegister")}</a></span>
                </div>
            </div>
        </#if>
    </#if>

</@layout.registrationLayout>
