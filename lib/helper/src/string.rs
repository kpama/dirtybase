const DEFAULT_DEMARCATORS: &[char] = &[
    ' ', '_', '-', '/', '\\', '&', '.', ',', '\'', '\"', '/', '+', '=', '\n', '\r', '\t',
];

pub fn to_pascal_case(subject: &str) -> String {
    return to_pascal_case_with(subject, DEFAULT_DEMARCATORS.iter());
}

pub fn to_pascal_case_with<D: Iterator<Item = &'static char>>(
    subject: &str,
    demarcators: D,
) -> String {
    let mut raw = subject.to_string();
    for demarcator in demarcators {
        raw = raw.split(*demarcator).fold("".to_string(), |mut acc, p| {
            let mut s = p.to_string();
            acc.push_str(&format!("{}{s}", s.remove(0).to_uppercase()));
            acc
        });
    }

    print!("stop: {}", &raw);
    raw
}

pub fn to_camel_case(subject: &str) -> String {
    let mut raw = to_pascal_case(subject);
    format!("{}{raw}", raw.remove(0).to_lowercase())
}

pub fn to_camel_case_with<D: Iterator<Item = &'static char>>(
    subject: &str,
    demarcators: D,
) -> String {
    let mut raw = to_pascal_case_with(subject, demarcators);
    format!("{}{raw}", raw.remove(0).to_lowercase())
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_pascal() {
        for d in DEFAULT_DEMARCATORS.iter() {
            let subject = format!("foo{d}bar{d}bass");
            assert_eq!(
                to_pascal_case(&subject),
                "FooBarBass",
                "{} suppose to be {}",
                subject,
                "FooBarBass"
            );
        }
    }

    #[test]
    fn test_pascal_two() {
        let subject = "my first test";
        assert_eq!(
            to_pascal_case(subject),
            "MyFirstTest",
            "{} suppose to be {}",
            subject,
            "MyFirstTest"
        );
    }

    #[test]
    fn test_pascal_with() {
        let subject = "fookbarkbass".to_string();
        let d = &['k'];
        assert_eq!(
            to_pascal_case_with(&subject, d.iter()),
            "FooBarBass",
            "{} suppose to be {}",
            subject,
            "FooBarBass"
        );
    }
}
