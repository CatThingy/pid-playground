use eframe::egui::plot::Value;

pub struct Model {
    pub accel: f64,
    pub vel: f64,

    pub value: f64,
    pub setpoint: f64,

    pub elapsed_time: f64,

    pub controller: PidController,
}

pub struct PidController {
    pub k_p: f64,
    pub k_i: f64,
    pub k_d: f64,

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
            damping: 0.5,
            applied_force: 0.0,
            timestep: 0.016,
            max_accel: 10.0,
        }
    }
}

impl Default for PidController {
    fn default() -> Self {
        Self {
            k_p: 0.0,
            k_i: 0.0,
            k_d: 0.0,

            prev_error: 0.0,
            integral: 0.0,
        }
    }
}

impl Default for Model {
    fn default() -> Self {
        Self {
            accel: 0.0,
            vel: 0.0,
            value: 0.0,
            setpoint: 100.0,
            elapsed_time: 0.0,

            controller: PidController::default(),
        }
    }
}

impl PidController {
    pub fn reset(&mut self) {
        self.prev_error = 0.0;
        self.integral = 0.0;
    }
    pub fn update(&mut self, setpoint: f64, value: f64, d_t: f64) -> f64 {
        let error = setpoint - value;
        let derivative = (error - self.prev_error) / d_t;

        self.prev_error = error;
        self.integral += error * d_t;

        return error * self.k_p + self.integral * self.k_i + derivative * self.k_d;
    }
}

impl Model {
    pub fn reset(&mut self) {
        self.controller.reset();
        self.accel = 0.0;
        self.vel = 0.0;
        self.value = 0.0;
        self.elapsed_time = 0.0;
    }

    pub fn update(&mut self, env: &Environment, d_t: f64) {
        self.accel = self.controller.update(self.setpoint, self.value, d_t)
            .clamp(-env.max_accel, env.max_accel)
            - self.vel * env.damping
            + env.applied_force;

        self.vel += self.accel * d_t;
        self.value += self.vel * d_t;

        self.elapsed_time += d_t;
    }

    pub fn evaluate(&mut self, time: f64, env: &Environment) -> Vec<Value> {
        let mut result = Vec::<Value>::new();
        let start_time = self.elapsed_time;

        loop {
            self.update(env, env.timestep);

            result.push(Value {
                x: self.elapsed_time,
                y: self.value,
            });
            if self.elapsed_time - start_time > time {
                break;
            }
        }

        return result;
    }
}
