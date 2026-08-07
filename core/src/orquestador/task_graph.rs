// 🔱 TASK GRAPH — Grafo de ejecución acíclico dirigido (DAG) de tareas atómicas
// Puro Rust, sin unwrap(), sin expect(), listo para producción.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Prioridad de una tarea atómica
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Critical,
    High,
    Normal,
    Low,
}

/// Acción concreta que la herramienta ejecutará en el Sandbox
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "action", content = "params")]
pub enum ToolAction {
    #[serde(rename = "read_file")]
    ReadFile { target: String },
    #[serde(rename = "write_file")]
    WriteFile { target: String, payload: String },
    #[serde(rename = "execute_cmd")]
    ExecuteCmd { target: String },
    #[serde(rename = "search_code")]
    SearchCode { target: String, pattern: Option<String> },
    #[serde(rename = "list_dir")]
    ListDir { target: String },
    #[serde(rename = "noop")]
    Noop,
}

/// Estado actual de un nodo/tarea en el grafo
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeState {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
    Blocked,
}

/// Un nodo del grafo de ejecución — una tarea atómica e indivisible
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNode {
    pub id: String,                         // Ej: "step_01"
    pub instruction: String,                // Ej: "Leer el archivo core/src/lib.rs"
    pub tool: ToolAction,                   // Acción sugerida inicial
    pub depends_on: Vec<String>,            // IDs de nodos de los que depende
    pub output_schema: serde_json::Value,   // Formato estricto esperado (JSON Schema)
    pub max_retries: u8,                    // Default: 2
    pub priority: Priority,                 // Prioridad
    pub state: NodeState,                   // Estado actual
    pub error_msg: Option<String>,          // Mensaje en caso de fallo
}

/// Estado global del DAG
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DAGState {
    Idle,
    Running,
    Paused,
    Completed,
    Failed,
}

/// El Grafo Acíclico Dirigido que contiene el plan de ejecución completo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDAG {
    pub id: String,                         // Identificador único de ejecución
    pub objective: String,                  // Objetivo de alto nivel del Arquitecto
    pub nodes: HashMap<String, TaskNode>,    // Mapa id -> nodo
    pub state: DAGState,                    // Estado global del DAG
}

impl TaskDAG {
    /// Crea un nuevo grafo de ejecución vacío
    pub fn new(id: String, objective: String) -> Self {
        Self {
            id,
            objective,
            nodes: HashMap::new(),
            state: DAGState::Idle,
        }
    }

    /// Añade una tarea/nodo al grafo
    pub fn add_node(&mut self, node: TaskNode) {
        self.nodes.insert(node.id.clone(), node);
        self.recalculate_states();
    }

    /// Añade una dependencia de dependiente -> depende_de
    pub fn add_dependency(&mut self, dependent: &str, depends_on: &str) -> bool {
        if dependent == depends_on {
            return false; // Evitar la autodependencia directa
        }

        // Verificar que ambos nodos existan
        if !self.nodes.contains_key(dependent) || !self.nodes.contains_key(depends_on) {
            return false;
        }

        // Detectar si añadir esta dependencia crearía un ciclo
        if self.would_create_cycle(dependent, depends_on) {
            return false;
        }

        if let Some(node) = self.nodes.get_mut(dependent) {
            if !node.depends_on.contains(&depends_on.to_string()) {
                node.depends_on.push(depends_on.to_string());
            }
        }

        self.recalculate_states();
        true
    }

    /// Comprueba recursivamente si añadir una arista dependent -> depends_on crearía un ciclo
    fn would_create_cycle(&self, start: &str, target: &str) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![target.to_string()];

        while let Some(current) = stack.pop() {
            if current == start {
                return true; // Encontró camino de regreso al origen
            }
            if visited.insert(current.clone()) {
                if let Some(node) = self.nodes.get(&current) {
                    for dep in &node.depends_on {
                        stack.push(dep.clone());
                    }
                }
            }
        }
        false
    }

    /// Cambia el estado de un nodo específico y recalcula estados del grafo
    pub fn update_node_state(&mut self, id: &str, new_state: NodeState, error_msg: Option<String>) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.state = new_state;
            node.error_msg = error_msg;
        }
        self.recalculate_states();
    }

    /// Devuelve los nodos listos para ser paralelizados o ejecutados (Ready)
    pub fn get_ready_nodes(&self) -> Vec<TaskNode> {
        let mut ready = Vec::new();
        for node in self.nodes.values() {
            if node.state == NodeState::Ready {
                ready.push(node.clone());
            }
        }
        // Ordenar por prioridad (Critical -> High -> Normal -> Low)
        ready.sort_by(|a, b| a.priority.cmp(&b.priority));
        ready
    }

    /// Recalcula el estado de todos los nodos según sus dependencias e impacto en cascada
    pub fn recalculate_states(&mut self) {
        let mut completed_ids = HashSet::new();
        let mut failed_ids = HashSet::new();

        // 1. Recopilar estados inmutables
        for node in self.nodes.values() {
            match node.state {
                NodeState::Completed => {
                    completed_ids.insert(node.id.clone());
                }
                NodeState::Failed => {
                    failed_ids.insert(node.id.clone());
                }
                _ => {}
            }
        }

        // 2. Resolver estados dinámicos (Pending, Ready, Blocked) de forma iterativa
        let node_ids: Vec<String> = self.nodes.keys().cloned().collect();
        for id in node_ids {
            let mut state_to_apply = None;

            if let Some(node) = self.nodes.get(&id) {
                // No tocar nodos ejecutándose, completados o fallidos
                if node.state == NodeState::Running || node.state == NodeState::Completed || node.state == NodeState::Failed {
                    continue;
                }

                // Evaluar dependencias
                let mut all_completed = true;
                let mut any_failed = false;

                for dep in &node.depends_on {
                    if failed_ids.contains(dep) {
                        any_failed = true;
                        break;
                    }
                    if !completed_ids.contains(dep) {
                        all_completed = false;
                    }
                }

                if any_failed {
                    state_to_apply = Some(NodeState::Blocked);
                } else if all_completed {
                    state_to_apply = Some(NodeState::Ready);
                } else {
                    state_to_apply = Some(NodeState::Pending);
                }
            }

            if let Some(state) = state_to_apply {
                if let Some(node) = self.nodes.get_mut(&id) {
                    node.state = state;
                }
            }
        }

        // 3. Recalcular estado global del DAG
        let mut total = self.nodes.len();
        let mut done = 0;
        let mut running = 0;
        let mut failed = 0;

        for node in self.nodes.values() {
            match node.state {
                NodeState::Completed => done += 1,
                NodeState::Running => running += 1,
                NodeState::Failed => failed += 1,
                _ => {}
            }
        }

        if total == 0 {
            self.state = DAGState::Idle;
        } else if failed > 0 {
            self.state = DAGState::Failed;
        } else if done == total {
            self.state = DAGState::Completed;
        } else if running > 0 || done > 0 {
            self.state = DAGState::Running;
        } else {
            self.state = DAGState::Idle;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dag_creation_and_dependencies() {
        let mut dag = TaskDAG::new("test_dag_1".to_string(), "Refactorizar código".to_string());
        
        let step1 = TaskNode {
            id: "step_1".to_string(),
            instruction: "Leer archivo de configuracion".to_string(),
            tool: ToolAction::ReadFile { target: "config.json".to_string() },
            depends_on: vec![],
            output_schema: serde_json::Value::Null,
            max_retries: 2,
            priority: Priority::High,
            state: NodeState::Pending,
            error_msg: None,
        };

        let step2 = TaskNode {
            id: "step_2".to_string(),
            instruction: "Escribir nueva configuracion".to_string(),
            tool: ToolAction::WriteFile { 
                target: "config_new.json".to_string(), 
                payload: "{}".to_string() 
            },
            depends_on: vec![],
            output_schema: serde_json::Value::Null,
            max_retries: 2,
            priority: Priority::Normal,
            state: NodeState::Pending,
            error_msg: None,
        };

        dag.add_node(step1);
        dag.add_node(step2);

        // El step_1 y step_2 deberían estar listos (sin dependencias)
        let ready = dag.get_ready_nodes();
        assert_eq!(ready.len(), 2);
        assert_eq!(ready[0].id, "step_1"); // Mayor prioridad (High) viene primero

        // Añadir dependencia step_2 -> step_1
        assert!(dag.add_dependency("step_2", "step_1"));

        // Ahora solo step_1 debería estar listo
        let ready_after = dag.get_ready_nodes();
        assert_eq!(ready_after.len(), 1);
        assert_eq!(ready_after[0].id, "step_1");

        // Completar step_1
        dag.update_node_state("step_1", NodeState::Completed, None);

        // Ahora step_2 está listo
        let ready_final = dag.get_ready_nodes();
        assert_eq!(ready_final.len(), 1);
        assert_eq!(ready_final[0].id, "step_2");
    }

    #[test]
    fn test_dag_cycle_prevention() {
        let mut dag = TaskDAG::new("cycle_test".to_string(), "Evitar bucles".to_string());
        
        let a = TaskNode {
            id: "A".to_string(),
            instruction: "Task A".to_string(),
            tool: ToolAction::Noop,
            depends_on: vec![],
            output_schema: serde_json::Value::Null,
            max_retries: 2,
            priority: Priority::Normal,
            state: NodeState::Pending,
            error_msg: None,
        };

        let b = TaskNode {
            id: "B".to_string(),
            instruction: "Task B".to_string(),
            tool: ToolAction::Noop,
            depends_on: vec![],
            output_schema: serde_json::Value::Null,
            max_retries: 2,
            priority: Priority::Normal,
            state: NodeState::Pending,
            error_msg: None,
        };

        dag.add_node(a);
        dag.add_node(b);

        assert!(dag.add_dependency("A", "B")); // A depende de B
        assert!(!dag.add_dependency("B", "A")); // B depende de A crearía un ciclo! -> Debe retornar false
    }
}
