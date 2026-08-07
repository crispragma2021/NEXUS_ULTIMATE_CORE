use crate::sentidos::propiocepcion::EstadoSistema;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::BinaryHeap;

/// Experiencia para el entrenamiento de la red
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Experience {
    pub state: EstadoSistema,
    pub action: usize,
    pub reward: f64,
    pub next_state: EstadoSistema,
    pub done: bool,
}

#[derive(Clone, Debug)]
pub struct PrioritizedExperience {
    pub experience: Experience,
    pub priority: f64,
}

impl PartialEq for PrioritizedExperience {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}
impl Eq for PrioritizedExperience {}
impl Ord for PrioritizedExperience {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority
            .partial_cmp(&other.priority)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}
impl PartialOrd for PrioritizedExperience {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Buffer de Replay Priorizado (Prioritized Replay Buffer)
pub struct PrioritizedReplayBuffer {
    pub buffer: BinaryHeap<PrioritizedExperience>,
    pub max_size: usize,
    pub alpha: f64,
}

impl PrioritizedReplayBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            buffer: BinaryHeap::with_capacity(max_size),
            max_size,
            alpha: 0.6,
        }
    }

    pub fn push(&mut self, experience: Experience, td_error: f64) {
        let priority = (td_error.abs() + 1e-6).powf(self.alpha);
        if self.buffer.len() >= self.max_size {
            self.buffer.pop();
        }
        self.buffer.push(PrioritizedExperience {
            experience,
            priority,
        });
    }

    pub fn sample(&mut self, batch_size: usize) -> Vec<Experience> {
        let mut temp_buffer: Vec<PrioritizedExperience> = self.buffer.drain().collect();
        if temp_buffer.len() < batch_size {
            // Volver a insertar si no hay suficientes elementos
            for exp in temp_buffer.into_iter() {
                self.buffer.push(exp);
            }
            return Vec::new();
        }

        let mut rng = rand::thread_rng();
        temp_buffer.shuffle(&mut rng);

        let sampled: Vec<Experience> = temp_buffer
            .iter()
            .take(batch_size)
            .map(|pe| pe.experience.clone())
            .collect();

        for exp in temp_buffer.into_iter().skip(batch_size) {
            self.buffer.push(exp);
        }

        sampled
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

/// Dueling Q-Network (Red Neuronal Pura en Rust)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DuelingQNetwork {
    pub w_common: Vec<Vec<f64>>,
    pub b_common: Vec<f64>,
    pub w_advantage: Vec<Vec<f64>>,
    pub b_advantage: Vec<f64>,
    pub w_value: Vec<Vec<f64>>,
    pub b_value: Vec<f64>,
}

impl DuelingQNetwork {
    pub fn new(input_size: usize, hidden_size: usize, output_size: usize) -> Self {
        let mut rng = rand::thread_rng();
        let bound_common = (6.0 / (input_size + hidden_size) as f64).sqrt();
        let bound_adv = (6.0 / (hidden_size + output_size) as f64).sqrt();
        let bound_val = (6.0 / (hidden_size + 1) as f64).sqrt();

        use rand::Rng;

        let w_common = (0..hidden_size)
            .map(|_| {
                (0..input_size)
                    .map(|_| rng.gen_range(-bound_common..bound_common))
                    .collect()
            })
            .collect();
        let b_common = vec![0.0; hidden_size];

        let w_advantage = (0..output_size)
            .map(|_| {
                (0..hidden_size)
                    .map(|_| rng.gen_range(-bound_adv..bound_adv))
                    .collect()
            })
            .collect();
        let b_advantage = vec![0.0; output_size];

        let w_value = (0..1)
            .map(|_| {
                (0..hidden_size)
                    .map(|_| rng.gen_range(-bound_val..bound_val))
                    .collect()
            })
            .collect();
        let b_value = vec![0.0; 1];

        Self {
            w_common,
            b_common,
            w_advantage,
            b_advantage,
            w_value,
            b_value,
        }
    }

    pub fn forward(&self, input: &[f64]) -> Vec<f64> {
        let hidden = self.forward_common(input);
        let advantage = self.forward_advantage(&hidden);
        let value = self.forward_value(&hidden)[0];

        let mean_advantage: f64 = advantage.iter().sum::<f64>() / advantage.len() as f64;
        advantage
            .iter()
            .map(|&a| value + a - mean_advantage)
            .collect()
    }

    pub fn forward_common(&self, input: &[f64]) -> Vec<f64> {
        let mut hidden = vec![0.0; self.b_common.len()];
        for i in 0..hidden.len() {
            let mut sum = self.b_common[i];
            for j in 0..input.len() {
                sum += self.w_common[i][j] * input[j];
            }
            hidden[i] = if sum > 0.0 { sum } else { 0.0 }; // ReLU
        }
        hidden
    }

    fn forward_advantage(&self, hidden: &[f64]) -> Vec<f64> {
        let mut advantage = vec![0.0; self.b_advantage.len()];
        for i in 0..advantage.len() {
            let mut sum = self.b_advantage[i];
            for j in 0..hidden.len() {
                sum += self.w_advantage[i][j] * hidden[j];
            }
            advantage[i] = sum;
        }
        advantage
    }

    fn forward_value(&self, hidden: &[f64]) -> Vec<f64> {
        let mut value = vec![0.0; self.b_value.len()];
        for i in 0..value.len() {
            let mut sum = self.b_value[i];
            for j in 0..hidden.len() {
                sum += self.w_value[i][j] * hidden[j];
            }
            value[i] = sum;
        }
        value
    }
}

/// Agente de Aprendizaje por Refuerzo (DQN)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DQNAgent {
    pub online_network: DuelingQNetwork,
    pub target_network: DuelingQNetwork,
    pub discount_factor: f64,
    pub epsilon: f64,
}

impl Default for DQNAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl DQNAgent {
    pub fn new() -> Self {
        let online = DuelingQNetwork::new(11, 64, 55);
        let target = online.clone();
        Self {
            online_network: online,
            target_network: target,
            discount_factor: 0.95,
            epsilon: 0.1,
        }
    }

    pub fn select_action(&self, input: &[f64]) -> usize {
        let mut rng = rand::thread_rng();
        use rand::Rng;
        if rng.gen::<f64>() < self.epsilon {
            rng.gen_range(0..55)
        } else {
            let q_values = self.online_network.forward(input);
            q_values
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(idx, _)| idx)
                .unwrap_or(0)
        }
    }

    pub fn train_step(&mut self, batch: &[Experience], lr: f64, tau: f64) -> f64 {
        let batch_size = batch.len();
        if batch_size == 0 {
            return 0.0;
        }

        let mut total_loss = 0.0;

        let mut grad_w_common = vec![vec![0.0; 11]; 64];
        let mut grad_b_common = vec![0.0; 64];
        let mut grad_w_adv = vec![vec![0.0; 64]; 55];
        let mut grad_b_adv = vec![0.0; 55];
        let mut grad_w_val = vec![vec![0.0; 64]; 1];
        let mut grad_b_val = [0.0; 1];

        for exp in batch {
            let input = exp.state.to_input_vector();
            let next_input = exp.next_state.to_input_vector();

            let q_values = self.online_network.forward(&input);
            let q_pred = q_values[exp.action];

            let target_q = if exp.done {
                exp.reward
            } else {
                let online_next_q = self.online_network.forward(&next_input);
                let best_next_action = online_next_q
                    .iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(idx, _)| idx)
                    .unwrap_or(0);

                let target_next_q = self.target_network.forward(&next_input);
                exp.reward + self.discount_factor * target_next_q[best_next_action]
            };

            let diff = q_pred - target_q;
            total_loss += diff * diff;

            let d = (2.0 * diff).clamp(-1.0, 1.0);

            let h = self.online_network.forward_common(&input);
            let n = q_values.len() as f64;

            let mut g_a = vec![0.0; 55];
            for i in 0..55 {
                let dq_da = if i == exp.action {
                    1.0 - 1.0 / n
                } else {
                    -1.0 / n
                };
                g_a[i] = d * dq_da;
            }

            let g_v = d;

            for i in 0..55 {
                grad_b_adv[i] += g_a[i];
                for j in 0..64 {
                    grad_w_adv[i][j] += g_a[i] * h[j];
                }
            }

            grad_b_val[0] += g_v;
            for j in 0..64 {
                grad_w_val[0][j] += g_v * h[j];
            }

            let mut g_h = vec![0.0; 64];
            for j in 0..64 {
                let mut sum_adv = 0.0;
                for i in 0..55 {
                    sum_adv += g_a[i] * self.online_network.w_advantage[i][j];
                }
                let sum_val = g_v * self.online_network.w_value[0][j];
                g_h[j] = sum_adv + sum_val;
            }

            let mut hidden_sum = vec![0.0; 64];
            for i in 0..64 {
                let mut sum = self.online_network.b_common[i];
                for j in 0..11 {
                    sum += self.online_network.w_common[i][j] * input[j];
                }
                hidden_sum[i] = sum;
            }

            let mut g_h_relu = vec![0.0; 64];
            for j in 0..64 {
                g_h_relu[j] = if hidden_sum[j] > 0.0 { g_h[j] } else { 0.0 };
            }

            for j in 0..64 {
                grad_b_common[j] += g_h_relu[j];
                for k in 0..11 {
                    grad_w_common[j][k] += g_h_relu[j] * input[k];
                }
            }
        }

        let scale = 1.0 / batch_size as f64;

        for i in 0..64 {
            self.online_network.b_common[i] -= lr * grad_b_common[i] * scale;
            for j in 0..11 {
                self.online_network.w_common[i][j] -= lr * grad_w_common[i][j] * scale;
            }
        }

        for i in 0..55 {
            self.online_network.b_advantage[i] -= lr * grad_b_adv[i] * scale;
            for j in 0..64 {
                self.online_network.w_advantage[i][j] -= lr * grad_w_adv[i][j] * scale;
            }
        }

        self.online_network.b_value[0] -= lr * grad_b_val[0] * scale;
        for j in 0..64 {
            self.online_network.w_value[0][j] -= lr * grad_w_val[0][j] * scale;
        }

        self.update_target(tau);

        total_loss / batch_size as f64
    }

    fn update_target(&mut self, tau: f64) {
        let lerp = |t: f64, o: f64| t * (1.0 - tau) + o * tau;

        for i in 0..64 {
            self.target_network.b_common[i] = lerp(
                self.target_network.b_common[i],
                self.online_network.b_common[i],
            );
            for j in 0..11 {
                self.target_network.w_common[i][j] = lerp(
                    self.target_network.w_common[i][j],
                    self.online_network.w_common[i][j],
                );
            }
        }

        for i in 0..55 {
            self.target_network.b_advantage[i] = lerp(
                self.target_network.b_advantage[i],
                self.online_network.b_advantage[i],
            );
            for j in 0..64 {
                self.target_network.w_advantage[i][j] = lerp(
                    self.target_network.w_advantage[i][j],
                    self.online_network.w_advantage[i][j],
                );
            }
        }

        self.target_network.b_value[0] = lerp(
            self.target_network.b_value[0],
            self.online_network.b_value[0],
        );
        for j in 0..64 {
            self.target_network.w_value[0][j] = lerp(
                self.target_network.w_value[0][j],
                self.online_network.w_value[0][j],
            );
        }
    }
}
