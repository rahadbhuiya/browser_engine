#[derive(Debug, Clone, Default)]
pub struct NavBarHistory {
    pub back_stack: Vec<String>,
    pub forward_stack: Vec<String>,
    pub current_url: Option<String>,
}

impl NavBarHistory {
    pub fn navigate_to(&mut self, url: &str) {
        if let Some(curr) = self.current_url.take() {
            self.back_stack.push(curr);
        }
        self.forward_stack.clear();
        self.current_url = Some(url.to_string());
    }

    pub fn can_go_back(&self) -> bool {
        !self.back_stack.is_empty()
    }

    pub fn can_go_forward(&self) -> bool {
        !self.forward_stack.is_empty()
    }

    pub fn go_back(&mut self) -> Option<String> {
        if let Some(prev) = self.back_stack.pop() {
            if let Some(curr) = self.current_url.take() {
                self.forward_stack.push(curr);
            }
            self.current_url = Some(prev.clone());
            Some(prev)
        } else {
            None
        }
    }

    pub fn go_forward(&mut self) -> Option<String> {
        if let Some(next) = self.forward_stack.pop() {
            if let Some(curr) = self.current_url.take() {
                self.back_stack.push(curr);
            }
            self.current_url = Some(next.clone());
            Some(next)
        } else {
            None
        }
    }
}
