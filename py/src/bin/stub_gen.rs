fn main() -> pyo3_stub_gen::Result<()> {
    let stub = deepwoken::stub_info()?;
    stub.generate()?;
    Ok(())
}
