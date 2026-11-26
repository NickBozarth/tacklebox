use serial_test::serial;
use tacklebox_client::comms::{CommunicationError, TokioSerialConnecion};
use tacklebox_core::comms::consts::BASE_NONCE_LEN;


fn new_conn() -> TokioSerialConnecion {
    TokioSerialConnecion::new("/dev/ttyACM0").unwrap()
}

async fn encrypt(
    conn: &mut TokioSerialConnecion,
    buf: &[u8]
) -> (Vec<u8>, [u8; BASE_NONCE_LEN]) {
    conn.encrypt_buffer(buf).await.expect("Failed to encrypt buf")
}

async fn decrypt(
    conn: &mut TokioSerialConnecion,
    buf: &[u8],
    nonce: [u8; BASE_NONCE_LEN]
) -> Vec<u8> {
    conn.decrypt_buffer(buf, nonce).await.expect("Failed to decrypt buf")
}




#[serial]
#[tokio::test]
async fn basic_enc_dec() {
    let mut conn = new_conn();
    let buf = [1, 2, 3];
    let (enc, nonce) = encrypt(&mut conn, &buf).await;
    let dec = decrypt(&mut conn, &enc, nonce).await;
    assert!(buf == dec.as_slice());
}


#[serial]
#[tokio::test]
async fn enc_dec_128_byte() {
    let mut conn = new_conn();
    let buf = b"
        this is a 128 byte long string used to test encryption and decryption.
        128 bytes is ecactly the buffer len
    ";
    let (enc, nonce) = encrypt(&mut conn, buf).await;
    let dec = decrypt(&mut conn, &enc, nonce).await;
    assert!(buf == dec.as_slice());
}

#[serial]
#[tokio::test]
async fn enc_dec_gt_128_byte() {
    let mut conn = new_conn();
    let buf = b"
        Buffers are digested in 128 byte blocks. This buffer is greater than
        128 bytes so it will be interpreted as 2 blocks to encrypt!
    ";
    let (enc, nonce) = encrypt(&mut conn, buf).await;
    let dec = decrypt(&mut conn, &enc, nonce).await;
    assert!(buf == dec.as_slice());
}

#[serial]
#[tokio::test]
#[ignore = "This takes a while (about 30s for me) but it should pass if all others do"]
async fn enc_dec_lorem_5k() {
    let mut conn = new_conn();
    let lorem = include_bytes!("./test_files/lorem_5k.txt");
    let (enc, nonce) = encrypt(&mut conn, lorem).await;
    let dec = decrypt(&mut conn, &enc, nonce).await;
    assert!(lorem == dec.as_slice());
}
