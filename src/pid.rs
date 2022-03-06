use eframe::egui::plot::Value;

pub struct PidController {
    pub env: Environment,

    pub k_p: f64,
    pub k_i: f64,
    pub k_d: f64,

    pub accel: f64,
    pub vel: f64,

    pub value: f64,
    pub setpoint: f64,

    pub elapsed_time: f64,

    prev_error: f64,
    integral: f64,
}

pub struct Environment {
    pub damping: f64,
    pub applied_force: f64,
    pub timestep: f64,
    pub max_accel: f64,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            damping: 1.0,
            applied_force: 0.0,
            timestep: 0.1,
            max_accel: 10.0,
        }
    }
}

impl Default for PidController {
    fn default() -> Self {
        Self {
            env: Environment::default(),
            k_p: 0.0,
            k_i: 0.0,
            k_d: 0.0,

            value: 0.0,
            setpoint: 100.0,

            elapsed_time: 0.0,

            accel: 0.0,
            vel: 0.0,

            prev_error: 0.0,
            integral: 0.0,
        }
    }
}

impl PidController {
    pub fn reset(&mut self) {
        self.accel = 0.0;
        self.vel = 0.0;
        self.value = 0.0;
        self.prev_error = 0.0;
        self.integral = 0.0;
        self.elapsed_time = 0.0;
    }

    pub fn update(&mut self, d_t: f64) {
        let error = self.setpoint - self.value;
        let derivative = (error - self.prev_error) / d_t;

        self.prev_error = error;
        self.integral += error * d_t;

        self.accel = (error * self.k_p + self.integral * self.k_i + derivative * self.k_d).clamp(-self.env.max_accel, self.env.max_accel)
            - self.vel * self.env.damping
            + self.env.applied_force;

        self.vel += self.accel * d_t;
        self.value += self.vel * d_t;

        self.elapsed_time += d_t;
    }

    pub fn evaluate(&mut self, time: f64) -> Vec<Value> {
        let mut result = Vec::<Value>::new();
        let start_time = self.elapsed_time;

        loop {
            if self.elapsed_time - start_time > time {
                break;
            }
            self.update(self.env.timestep);

            result.push(Value {
                x: self.elapsed_time,
                y: self.value,
            });
        }
        return result;
    }
}
