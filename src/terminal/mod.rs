

pub struct LocalPty {
    pty: alacritty_terminal::tty::Pty,
}

impl LocalPty {
    pub fn new() -> anyhow::Result<Self> {
        let config = alacritty_terminal::config::PtyConfig::default();
        let window_size = alacritty_terminal::event::WindowSize{
            num_cols: 80, num_lines: 20, cell_height: 1, cell_width: 1
        };
        let pty = alacritty_terminal::tty::new(&config, window_size, 0)?;
        Ok(Self {pty} )
    }

    pub fn resize(&mut self, width: u16, height: u16) -> anyhow::Result<()> {
        use alacritty_terminal::event::OnResize;
        let window_size = alacritty_terminal::event::WindowSize{
            num_cols: width, num_lines: height, cell_height: 1, cell_width: 1
        };
        self.pty.on_resize(window_size);
        Ok(())
    }

}
