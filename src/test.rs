#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_dictionary_some() {
        let dict =
            Dictionary::new_from_file("mappings.json").expect("Dictionary initialization failed");
        assert_eq!(
            dict.resolve_callback("some_id"),
            Some(&"https://google.com".to_string())
        );
        assert_eq!(
            dict.resolve_callback("nyan"),
            Some(&"http://www.nyan.cat/".to_string())
        );
    }

    #[test]
    fn test_dictionary_none() {
        let dict =
            Dictionary::new_from_file("mappings.json").expect("Dictionary initialization failed");
        assert_eq!(dict.resolve_callback("none_id"), None);
    }

    #[test]
    fn test_dictionary_err() {
        assert!(Dictionary::new_from_file("mappings").is_err());
    }

}
