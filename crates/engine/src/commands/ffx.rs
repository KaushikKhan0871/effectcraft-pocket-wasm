//! Defensive reader for Adobe After Effects RIFX/FaFX preset containers.
//!
//! This module deliberately separates container decoding from effect translation. FFX
//! parameter payloads are plug-in-specific; callers must not guess their meaning.

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Chunk<'a> {
    pub id: [u8; 4],
    pub kind: Option<[u8; 4]>,
    pub data: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EffectBlock {
    pub name: String,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FfxDocument {
    pub effects: Vec<EffectBlock>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FfxError {
    BadHeader,
    Truncated,
    InvalidChunk(&'static str),
    NoEffectBlocks,
}

fn be_u32(b: &[u8]) -> Result<usize, FfxError> {
    let a: [u8; 4] = b.get(..4).ok_or(FfxError::Truncated)?.try_into().map_err(|_| FfxError::Truncated)?;
    Ok(u32::from_be_bytes(a) as usize)
}

/// Read IFF chunks in a RIFX container. LIST's first four data bytes are its list type.
pub(crate) fn chunks(data: &[u8]) -> Result<Vec<Chunk<'_>>, FfxError> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < data.len() {
        if data.len() - pos < 8 { return Err(FfxError::Truncated); }
        let id: [u8; 4] = data[pos..pos + 4].try_into().map_err(|_| FfxError::Truncated)?;
        let len = be_u32(&data[pos + 4..pos + 8])?;
        let start = pos.checked_add(8).ok_or(FfxError::Truncated)?;
        let end = start.checked_add(len).ok_or(FfxError::Truncated)?;
        if end > data.len() { return Err(FfxError::Truncated); }
        let raw = &data[start..end];
        let (kind, body) = if id == *b"LIST" || id == *b"RIFF" || id == *b"RIFX" {
            if raw.len() < 4 { return Err(FfxError::InvalidChunk("container without type")); }
            (Some(raw[..4].try_into().map_err(|_| FfxError::Truncated)?), &raw[4..])
        } else { (None, raw) };
        out.push(Chunk { id, kind, data: body });
        pos = end.checked_add(len & 1).ok_or(FfxError::Truncated)?;
        if pos > data.len() { return Err(FfxError::Truncated); }
    }
    Ok(out)
}

fn descendants<'a>(data: &'a [u8], wanted: [u8; 4], out: &mut Vec<&'a [u8]>) -> Result<(), FfxError> {
    for c in chunks(data)? {
        if c.id == wanted { out.push(c.data); }
        if c.id == *b"LIST" { descendants(c.data, wanted, out)?; }
    }
    Ok(())
}

fn decode_fnam(data: &[u8]) -> Option<String> {
    // Adobe's fnam payload: "Utf8", a big-endian byte count, then UTF-8 bytes.
    if data.len() < 8 || &data[..4] != b"Utf8" { return None; }
    let n = be_u32(&data[4..8]).ok()?;
    let s = data.get(8..8usize.checked_add(n)?)?;
    Some(String::from_utf8_lossy(s).trim_end_matches('\0').to_string())
}

pub(crate) fn parse(bytes: &[u8]) -> Result<FfxDocument, FfxError> {
    if bytes.len() < 12 || &bytes[..4] != b"RIFX" || &bytes[8..12] != b"FaFX" { return Err(FfxError::BadHeader); }
    let declared = be_u32(&bytes[4..8])?;
    if declared.checked_add(8).ok_or(FfxError::Truncated)? > bytes.len() { return Err(FfxError::Truncated); }
    let root = chunks(&bytes[12..8 + declared])?;
    let mut effects = Vec::new();
    for list in root.iter().filter(|c| c.id == *b"LIST" && c.kind == Some(*b"besc")) {
        for block in chunks(list.data)?.into_iter().filter(|c| c.id == *b"LIST" && c.kind == Some(*b"sspc")) {
            let mut names = Vec::new();
            descendants(block.data, *b"fnam", &mut names)?;
            let name = names.first().and_then(|v| decode_fnam(v)).unwrap_or_else(|| "Unknown FFX effect".into());
            effects.push(EffectBlock { name, payload: block.data.to_vec() });
        }
    }
    if effects.is_empty() { return Err(FfxError::NoEffectBlocks); }
    Ok(FfxDocument { effects })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_non_ffx_and_truncated_headers() {
        assert_eq!(parse(b"not an ffx"), Err(FfxError::BadHeader));
        assert_eq!(parse(b"RIFX"), Err(FfxError::BadHeader));
    }
    #[test]
    fn reads_the_supplied_preset_structure() {
        // Structural fixture: RIFX/FaFX containing LIST/besc -> LIST/sspc -> fnam.
        let mut body = Vec::new();
        body.extend_from_slice(b"fnam"); body.extend_from_slice(&12u32.to_be_bytes());
        body.extend_from_slice(b"Utf8"); body.extend_from_slice(&4u32.to_be_bytes()); body.extend_from_slice(b"Test");
        let mut sspc = b"sspc".to_vec(); sspc.extend_from_slice(&(body.len() as u32).to_be_bytes()); sspc.extend_from_slice(&body);
        let mut list = b"LIST".to_vec(); list.extend_from_slice(&(sspc.len() as u32).to_be_bytes()); list.extend_from_slice(&sspc);
        let mut besc = b"besc".to_vec(); besc.extend_from_slice(&list);
        let mut root = b"LIST".to_vec(); root.extend_from_slice(&(besc.len() as u32).to_be_bytes()); root.extend_from_slice(&besc);
        let mut file = b"RIFX".to_vec(); file.extend_from_slice(&(4u32 + root.len() as u32).to_be_bytes()); file.extend_from_slice(b"FaFX"); file.extend_from_slice(&root);
        let parsed = parse(&file).unwrap();
        assert_eq!(parsed.effects.len(), 1);
        assert_eq!(parsed.effects[0].name, "Test");
    }
}
