//! F1-T20: kunci job tunggal `pg_try_advisory_lock` tingkat sesi di koneksi khusus (KEPUTUSAN-74,
//! KEPUTUSAN-103, docs/05-physical-schema.md §6.4). PostgreSQL 18 sekali pakai dari
//! `scripts/db-ephemeral.sh`.
//!
//! Database efemeral milik satu kali verifikasi; setiap test di berkas ini memakai kunci registri yang
//! berbeda, sehingga test yang berjalan paralel tidak saling memegang kunci yang sama.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use common::*;
use platform_jobs::lock::{RS_AUDIT_RELAY, RS_KEYCLOAK_EVENT_PULLER, RS_PRIVACY_WORKER};
use platform_jobs::{LockAttempt, SingletonLock};
use sqlx::{Connection, PgConnection};
use std::time::Duration as StdDuration;

async fn acquire(conn: PgConnection, key: platform_jobs::LockKey) -> SingletonLock {
    match SingletonLock::try_acquire(conn, key).await.unwrap() {
        LockAttempt::Acquired(l) => l,
        LockAttempt::Busy(_) => panic!("kunci {key} seharusnya bebas"),
    }
}

async fn expect_busy(conn: PgConnection, key: platform_jobs::LockKey) -> PgConnection {
    match SingletonLock::try_acquire(conn, key).await.unwrap() {
        LockAttempt::Acquired(_) => panic!("kunci {key} dipegang dua sesi sekaligus"),
        LockAttempt::Busy(c) => c,
    }
}

#[tokio::test]
async fn second_session_is_busy_until_holder_releases() {
    // Kunci relay audit RS: dua instans relay, hanya satu yang aktif (§6.4, test dua relay F1-T24).
    let key = RS_AUDIT_RELAY;
    let mut holder = acquire(dedicated_connection().await, key).await;
    let standby = expect_busy(dedicated_connection().await, key).await;
    // Siaga mencoba ulang berkala dengan koneksi yang sama: tetap ditolak selama pemegang hidup.
    let standby = expect_busy(standby, key).await;

    // Pekerjaan job tunggal berjalan lewat koneksi pemegang kunci.
    let one: i32 = sqlx::query_scalar!(r#"SELECT 1 AS "one!""#)
        .fetch_one(holder.connection())
        .await
        .unwrap();
    assert_eq!(one, 1);
    assert_eq!(holder.key(), key);

    let holder_conn = holder.release().await.unwrap();
    let new_holder = acquire(standby, key).await;
    // Bekas pemegang kini siaga.
    let holder_conn = expect_busy(holder_conn, key).await;
    new_holder.release().await.unwrap().close().await.unwrap();
    holder_conn.close().await.unwrap();
}

#[tokio::test]
async fn lock_is_released_when_holder_connection_ends() {
    // Koneksi pemegang putus (proses mati) = kunci lepas (pg_advisory_unlock_all saat sesi berakhir).
    let key = RS_PRIVACY_WORKER;
    let holder = acquire(dedicated_connection().await, key).await;
    let mut standby = expect_busy(dedicated_connection().await, key).await;
    drop(holder);

    // Server mengakhiri sesi secara asinkron setelah soket ditutup: coba ulang sampai 10 detik.
    let mut acquired = None;
    for _ in 0..100 {
        match SingletonLock::try_acquire(standby, key).await.unwrap() {
            LockAttempt::Acquired(l) => {
                acquired = Some(l);
                break;
            }
            LockAttempt::Busy(c) => {
                standby = c;
                tokio::time::sleep(StdDuration::from_millis(100)).await;
            }
        }
    }
    let lock = acquired.expect("kunci tidak lepas setelah koneksi pemegang ditutup");
    lock.release().await.unwrap().close().await.unwrap();
}

#[tokio::test]
async fn different_keys_are_independent_and_session_sees_its_own_lock() {
    let a = acquire(dedicated_connection().await, RS_KEYCLOAK_EVENT_PULLER).await;
    // Kunci lain di sesi lain tetap bebas.
    let other = dedicated_connection().await;
    let b = acquire(
        other,
        platform_jobs::lock::RME_KEYCLOAK_EVENT_PULLER_RESERVED,
    )
    .await;
    // Pemegang kunci terlihat di pg_locks sebagai kunci advisory milik backend-nya.
    let mut a = a;
    let held: bool = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM pg_locks
                           WHERE locktype = 'advisory' AND pid = pg_backend_pid() AND granted) AS "held!""#
    )
    .fetch_one(a.connection())
    .await
    .unwrap();
    assert!(held);
    a.release().await.unwrap().close().await.unwrap();
    b.release().await.unwrap().close().await.unwrap();
}

#[tokio::test]
async fn releasing_twice_is_impossible_and_unheld_release_is_reported() {
    // Tipe memastikan release hanya sekali (self dikonsumsi). Sesi yang tidak memegang kunci dan mencoba
    // melepasnya mendapat false dari server; dibuktikan langsung dengan fungsi yang sama.
    let mut conn = dedicated_connection().await;
    let released: bool =
        sqlx::query_scalar!(r#"SELECT pg_advisory_unlock(7999999::bigint) AS "released!""#)
            .fetch_one(&mut conn)
            .await
            .unwrap();
    assert!(!released);
    conn.close().await.unwrap();
}
