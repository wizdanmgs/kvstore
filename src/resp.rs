use tokio::io::AsyncReadExt;
use tokio::net::tcp::OwnedReadHalf;

#[derive(Debug)]
pub enum RespValue {
    Array(Vec<RespValue>),
    BulkString(String),
}

// =========================================================
// Parse RESP Array (only supporting array of bulk strings)
// =========================================================
pub async fn parse(reader: &mut OwnedReadHalf) -> anyhow::Result<RespValue> {
    let mut prefix = [0u8; 1];

    // read first byte (*)
    reader.read_exact(&mut prefix[..]).await?;

    match prefix[0] {
        b'*' => parse_array(reader).await,
        _ => Err(anyhow::anyhow!("Unsupported RESP type")),
    }
}

pub async fn read_line(reader: &mut OwnedReadHalf) -> anyhow::Result<String> {
    let mut buffer: Vec<u8> = Vec::new();

    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte[..]).await?;
        buffer.push(byte[0]);

        // check for CRLF
        if buffer.len() >= 2 && &buffer[buffer.len() - 2..] == b"\r\n" {
            break;
        }
    }

    // remove CRLF
    buffer.truncate(buffer.len() - 2);

    Ok(String::from_utf8(buffer)?)
}

pub fn encode_simple_string(msg: &str) -> String {
    format!("+{}\r\n", msg)
}

pub fn encode_error(msg: &str) -> String {
    format!("-{}\r\n", msg)
}

pub fn encode_bulk_string(msg: &str) -> String {
    format!("${}\r\n{}\r\n", msg.len(), msg)
}

pub fn encode_null() -> String {
    "$-1\r\n".to_string()
}

async fn parse_array(reader: &mut OwnedReadHalf) -> anyhow::Result<RespValue> {
    // read array length
    let len_line = read_line(reader).await?;
    let len: usize = len_line.parse()?;

    let mut items = Vec::with_capacity(len);

    for _ in 0..len {
        let mut prefix = [0u8; 1];
        reader.read_exact(&mut prefix[..]).await?;

        if prefix[0] != b'$' {
            return Err(anyhow::anyhow!("Expected bulk string"));
        }

        // read bulk length
        let bulk_len_line = read_line(reader).await?;
        let bulk_len: usize = bulk_len_line.parse()?;

        // read bulk data
        let mut data = vec![0u8; bulk_len];
        reader.read_exact(&mut data).await?;

        // consume trailing CRLF
        let mut crlf = [0u8; 2];
        reader.read_exact(&mut crlf[..]).await?;

        items.push(RespValue::BulkString(String::from_utf8(data)?));
    }

    Ok(RespValue::Array(items))
}
