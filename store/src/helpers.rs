use merk::Merk;

/// Read a value from the Merk store; panicks if fails.
pub(crate) fn must_get(merk: &Merk, key: &[u8]) -> Option<Vec<u8>> {
    merk.get(key).unwrap_or_else(|err| {
        panic!(
            "[cw-store]: failed to read the value corresponing to key {} from Merk store: {}",
            hex::encode(key),
            err,
        );
    })
}
