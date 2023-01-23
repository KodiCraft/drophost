#[cfg(test)]
mod tests {
    
    

    

    use super::*;

    #[test]
    fn test_parse_entry() {
        use crate::types::Host;
        let entry = "127.0.0.1\tlocalhost";
        let host = Host::parse_entry(entry).unwrap();
        assert_eq!(host.hostname, "localhost");
        assert_eq!(host.ip, "127.0.0.1");
    }

    #[test]
    fn test_parse_entry_invalid() {
        use crate::types::Host;
        let entry = "888.888.888.888";
        let host = Host::parse_entry(entry);
        assert!(host.is_none());
    }

    #[test]
    fn test_parse_file() {
        use crate::parser::FileReader;
        use std::path::Path;
        use std::collections::HashMap;
        use crate::types::Host;
        let path = Path::new("tests/std_file.conf");
        let mut file = FileReader::new(path, HashMap::new());
        file.parse_all();
        let expected_output = vec![
            Host {
                ip: "127.0.0.1".to_string(),
                hostname: "localhost".to_string(),
            },
            Host {
                ip: "8.8.8.8".to_string(),
                hostname: "goog".to_string(),
            },
            Host {
                ip: "1234.1234.1234.1234".to_string(),
                hostname: "stacked".to_string(),
            },
            Host {
                ip: "1.1.1.1".to_string(),
                hostname: "should_appear".to_string(),
            }
        ];

        assert_eq!(file.hosts.hosts, expected_output);
        
    }

    #[test]
    fn test_fail_and_abort() {
        use crate::parser::FileReader;
        use std::path::Path;
        use std::collections::HashMap;
        let path = Path::new("tests/invalid_file.conf");
        let mut file = FileReader::new(path, HashMap::new());
        file.parse_all();
        assert_eq!(file.hosts.hosts.len(), 2);
    }

    #[test]
    #[should_panic]
    fn test_file_does_not_exist() {
        use crate::parser::FileReader;
        use std::path::Path;
        use std::collections::HashMap;
        let path = Path::new("tests/does_not_exist.conf");
        // Prevent the program from exiting
        let mut file = FileReader::new(path, HashMap::new());
        file.parse_all();
    }
}