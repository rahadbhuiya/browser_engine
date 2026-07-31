#[derive(Debug, Clone)]
pub struct Tab {
    pub id: usize,
    pub title: String,
    pub url: String,
    pub scroll_y: f32,
}

impl Tab {
    pub fn new(id: usize, title: &str, url: &str) -> Self {
        Tab {
            id,
            title: title.to_string(),
            url: url.to_string(),
            scroll_y: 0.0,
        }
    }
}

pub struct TabManager {
    pub tabs: Vec<Tab>,
    pub active_index: usize,
    next_id: usize,
}

impl TabManager {
    pub fn new() -> Self {
        let default_tab = Tab::new(1, "New Tab", "about:blank");
        TabManager {
            tabs: vec![default_tab],
            active_index: 0,
            next_id: 2,
        }
    }

    pub fn new_tab(&mut self, title: &str, url: &str) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.tabs.push(Tab::new(id, title, url));
        self.active_index = self.tabs.len() - 1;
        id
    }

    pub fn close_tab(&mut self, index: usize) {
        if self.tabs.len() > 1 && index < self.tabs.len() {
            self.tabs.remove(index);
            if self.active_index >= self.tabs.len() {
                self.active_index = self.tabs.len() - 1;
            }
        }
    }

    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_index]
    }

    pub fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_index]
    }

    pub fn switch_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_index = index;
        }
    }
}
