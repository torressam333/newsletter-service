use unicode_segmentation::UnicodeSegmentation;

//Tuple struct
pub struct SubscriberName(String);

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

impl SubscriberName {
    /// Returns an instance of SubscriberName if the input satisfies all our
    /// validation constraints on subscriber names otherwise it will PANIC!
    pub fn parse(name: String) -> SubscriberName {
        let is_empty_or_whitespace = name.trim().is_empty();

        // Some chars are actually composed of multiple bytes
        let is_too_long = name.graphemes(true).count() > 256;

        // Iterate over all chars to check if any of them are in the forbidden chars array
        let forbidden_chars = ['/', '(', ')', ',', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_chars = name.chars().any(|g| forbidden_chars.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_chars {
            panic!("{} is not a valid subscriber name", name);
        } else {
            Self(name)
        }
    }

    pub fn inner(self) -> String {
        // Caller gets inner string but they dont' have SubscriberName anymore.
        // That's b/c inner takes 'self' by value consumin git according to move semantics
        self.0
    }
}
