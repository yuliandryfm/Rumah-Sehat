//! Test integrasi persetujuan orang kedua lewat Control Group terhadap OpenBao 2.7.2 mode dev sungguhan
//! (F1-T65, US-208 AC4, ADR-025, SRC-253, SRC-670, SRC-671). Butuh Docker; tanpa Docker test ini GAGAL.
//!
//! Penyiapan meniru rancangan production: KV v2 `rs-admin/bootstrap/super-admin` dengan policy operator ber-
//! `control_group` (factor grup `rs-bootstrap-approvers`, satu persetujuan), dua identitas userpass terpisah
//! (operator dan approver) dengan entitas masing-masing.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod common;

use common::OpenBao;
use crypto::{ApprovalError, ControlGroupRequest, CryptoError, verify_control_group_approval};
use foundation::{Sensitive, Uuid};
use serde_json::Value;
use std::time::Duration;

const MOUNT: &str = "rs-admin";
const PATH: &str = "bootstrap/super-admin";
const PURPOSE: &str = "rs_super_admin_bootstrap";

const OPERATOR_POLICY: &str = r#"
path "rs-admin/data/bootstrap/super-admin" {
  capabilities = ["read"]
  control_group = {
    ttl = "1h"
    factor "approver" {
      identity {
        group_names = ["rs-bootstrap-approvers"]
        approvals = 1
      }
      controlled_capabilities = ["read"]
    }
  }
}
path "rs-admin/data/lain" { capabilities = ["read"] }
path "sys/control-group/request" { capabilities = ["update"] }
"#;

const APPROVER_POLICY: &str = r#"
path "sys/control-group/authorize" { capabilities = ["update"] }
path "sys/control-group/request" { capabilities = ["update"] }
"#;

struct Login {
    token: String,
    entity: Uuid,
}

fn json(out: &std::process::Output) -> Value {
    assert!(
        out.status.success(),
        "bao gagal: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}

fn login(bao: &OpenBao, user: &str, password: &str) -> Login {
    let v = json(&bao.bao(&[
        "login",
        "-method=userpass",
        "-format=json",
        "-no-store",
        &format!("username={user}"),
        &format!("password={password}"),
    ]));
    Login {
        token: v["auth"]["client_token"].as_str().unwrap().to_string(),
        entity: Uuid::parse_str(v["auth"]["entity_id"].as_str().unwrap()).unwrap(),
    }
}

struct Setup {
    bao: OpenBao,
    operator: Login,
    approver: Login,
    /// Approver lain yang BUKAN anggota grup approver.
    outsider: Login,
}

fn setup() -> Setup {
    let bao = OpenBao::start();
    bao.bao_ok(&["secrets", "enable", "-path", MOUNT, "-version=2", "kv"]);
    bao.bao_ok(&[
        "kv",
        "put",
        "-mount",
        MOUNT,
        PATH,
        &format!("purpose={PURPOSE}"),
    ]);
    bao.bao_ok(&["kv", "put", "-mount", MOUNT, "lain", "purpose=lain"]);
    bao.bao_ok(&["auth", "enable", "userpass"]);
    bao.bao_stdin(
        &["policy", "write", "rs-bootstrap-operator", "-"],
        OPERATOR_POLICY,
    );
    bao.bao_stdin(
        &["policy", "write", "rs-bootstrap-approver", "-"],
        APPROVER_POLICY,
    );
    for (u, p) in [
        ("operator", "rs-bootstrap-operator"),
        ("approver", "rs-bootstrap-approver"),
        ("outsider", "rs-bootstrap-approver"),
    ] {
        bao.bao_ok(&[
            "write",
            &format!("auth/userpass/users/{u}"),
            &format!("password=pw-{u}-uji"),
            &format!("policies={p}"),
        ]);
    }
    let operator = login(&bao, "operator", "pw-operator-uji");
    let approver = login(&bao, "approver", "pw-approver-uji");
    let outsider = login(&bao, "outsider", "pw-outsider-uji");
    bao.bao_ok(&[
        "write",
        "identity/group",
        "name=rs-bootstrap-approvers",
        &format!("member_entity_ids={}", approver.entity),
    ]);
    Setup {
        bao,
        operator,
        approver,
        outsider,
    }
}

/// Operator membaca path ber-Control Group: OpenBao menjawab token pembungkus + accessor.
fn request_access(s: &Setup, path: &str) -> (String, String) {
    let v = json(&s.bao.bao_as(
        &s.operator.token,
        &["read", "-format=json", &format!("{MOUNT}/data/{path}")],
    ));
    let wrap = &v["wrap_info"];
    (
        wrap["token"].as_str().expect("wrap_info.token").to_string(),
        wrap["accessor"].as_str().unwrap().to_string(),
    )
}

fn authorize(s: &Setup, who: &Login, accessor: &str) -> std::process::Output {
    s.bao.bao_as(
        &who.token,
        &[
            "write",
            "-format=json",
            "sys/control-group/authorize",
            &format!("accessor={accessor}"),
        ],
    )
}

fn request(s: &Setup, operator_token: &str, wrap: &str, accessor: &str) -> ControlGroupRequest {
    ControlGroupRequest {
        address: s.bao.address.clone(),
        operator_token: Sensitive::new(operator_token.to_string()),
        accessor: accessor.to_string(),
        wrapping_token: Sensitive::new(wrap.to_string()),
        kv_mount: MOUNT.into(),
        kv_path: PATH.into(),
        purpose: PURPOSE.into(),
        timeout: Duration::from_secs(10),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn second_party_approval_is_verified_and_consumed_once() {
    let s = setup();
    let (wrap, accessor) = request_access(&s, PATH);

    // Belum disetujui.
    let r = request(&s, &s.operator.token, &wrap, &accessor);
    assert_eq!(
        verify_control_group_approval(&r).await,
        Err(ApprovalError::NotApproved)
    );

    // Operator tidak dapat menyetujui permintaannya sendiri; orang di luar grup tidak menambah persetujuan.
    assert!(!authorize(&s, &s.operator, &accessor).status.success());
    let out = json(&authorize(&s, &s.outsider, &accessor));
    assert_eq!(out["data"]["approved"], Value::Bool(false));
    assert_eq!(
        verify_control_group_approval(&r).await,
        Err(ApprovalError::NotApproved)
    );

    // Approver anggota grup menyetujui.
    let out = json(&authorize(&s, &s.approver, &accessor));
    assert_eq!(out["data"]["approved"], Value::Bool(true));

    // Orang lain (bukan peminta) tidak dapat memakai persetujuan itu.
    let r_other = request(&s, &s.approver.token, &wrap, &accessor);
    assert_eq!(
        verify_control_group_approval(&r_other).await,
        Err(ApprovalError::RequesterMismatch)
    );

    let ok = verify_control_group_approval(&r).await.unwrap();
    assert_eq!(ok.operator_entity_id, s.operator.entity);
    // Hanya anggota grup factor yang tercatat sebagai penyetuju; panggilan authorize oleh pemegang hak di
    // luar grup tidak menambah entri (diamati, SRC-671).
    assert_eq!(ok.approver_entity_ids, vec![s.approver.entity]);
    assert!(!ok.approver_entity_ids.contains(&ok.operator_entity_id));

    // Persetujuan yang sama tidak dapat dipakai lagi (token pembungkus dicabut setelah dibuka).
    let again = verify_control_group_approval(&r).await;
    assert!(
        matches!(
            again,
            Err(ApprovalError::OpenBao(CryptoError::KmsRejected { .. }))
        ),
        "{again:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn approval_for_another_path_or_purpose_is_rejected() {
    let s = setup();
    let (wrap, accessor) = request_access(&s, PATH);
    assert!(json(&authorize(&s, &s.approver, &accessor))["data"]["approved"] == Value::Bool(true));

    let mut wrong_path = request(&s, &s.operator.token, &wrap, &accessor);
    wrong_path.kv_path = "lain".into();
    assert_eq!(
        verify_control_group_approval(&wrong_path).await,
        Err(ApprovalError::WrongPath)
    );

    let mut wrong_purpose = request(&s, &s.operator.token, &wrap, &accessor);
    wrong_purpose.purpose = "rs_lain".into();
    assert_eq!(
        verify_control_group_approval(&wrong_purpose).await,
        Err(ApprovalError::WrongPurpose)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn invalid_tokens_and_accessors_are_rejected_by_openbao() {
    let s = setup();
    let (wrap, accessor) = request_access(&s, PATH);
    let bad_operator = request(&s, "s.token-palsu", &wrap, &accessor);
    assert!(matches!(
        verify_control_group_approval(&bad_operator).await,
        Err(ApprovalError::OpenBao(CryptoError::KmsRejected { .. }))
    ));
    let bad_accessor = request(&s, &s.operator.token, &wrap, "accessor-palsu");
    assert!(matches!(
        verify_control_group_approval(&bad_accessor).await,
        Err(ApprovalError::OpenBao(CryptoError::KmsRejected { .. }))
    ));
    // Root token tidak terikat entitas: tidak dapat menjadi operator.
    let root = request(&s, &s.bao.token, &wrap, &accessor);
    assert_eq!(
        verify_control_group_approval(&root).await,
        Err(ApprovalError::RequesterMismatch)
    );
}
