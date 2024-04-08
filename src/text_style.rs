pub struct TextStyle<C> {
    /// Text color.
    pub text_color: Option<C>,
}

impl<C> TextStyle<C> {
    pub fn new(text_color: C) -> Self {
        Self {
            text_color: Some(text_color),
        }
    }
}
