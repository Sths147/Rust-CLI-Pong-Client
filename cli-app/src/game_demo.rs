pub struct Demo {
    pub ball_x: f64,
    pub ball_y: f64,
    pub ball_dx: f64,
    pub ball_dy: f64,
    pub paddle_left_y: f64,
    pub paddle_right_y: f64,
}

impl Demo {
    pub fn update(&mut self) {
        self.ball_x += self.ball_dx;
        self.ball_y += self.ball_dy;
        if self.ball_x <= 5.0 || self.ball_x >= 95.0 {
            self.ball_dx = -self.ball_dx;
        }
        if self.ball_y <= 0.0 || self.ball_y >= 100.0 {
            self.ball_dy = -self.ball_dy;
        }
        if self.ball_x < 50.0 {
            self.paddle_left_y += (self.ball_y - self.paddle_left_y - 5.0) * 0.12;
            if self.paddle_left_y > 90.0 {
                self.paddle_left_y = 90.0;
            } else if self.paddle_left_y < 1.0 {
                self.paddle_left_y = 1.0;
            }
        } else {
            self.paddle_right_y += (self.ball_y - self.paddle_right_y - 5.0) * 0.13;
            if self.paddle_right_y > 90.0 {
                self.paddle_right_y = 90.0;
            } else if self.paddle_right_y < 1.0 {
                self.paddle_right_y = 1.0;
            }
        }
    }
}

impl Default for Demo {
    fn default() -> Self {
        Demo {
            ball_x: 50.0,
            ball_y: 50.0,
            ball_dx: 1.0,
            ball_dy: 1.0,
            paddle_left_y: 50.0,
            paddle_right_y: 50.0,
        }
    }
}