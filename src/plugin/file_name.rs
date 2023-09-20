pub trait FileName {
    fn extract_file_name(&self) -> String;
}
impl FileName for String {
    fn extract_file_name(&self) -> String {
        self.as_str().extract_file_name()
    }
}
impl FileName for &str {
    fn extract_file_name(&self) -> String {
        let path = self.replace("\\", "/");
        match path.rfind('/') {
            Some(index) => path[(index + 1)..].to_string(),
            None => path,
        }
    }
}
