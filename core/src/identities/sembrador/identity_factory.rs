use serde::{Deserialize, Serialize};

// ─── Identidad ──────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identidad {
    pub id: String,
    pub nombre: String,
    pub apellido: String,
    pub segundo_apellido: Option<String>,
    pub email: String,
    pub password: String,
    pub recovery_email: Option<String>,
    pub fecha_nacimiento: String,
    pub pais: String,
    pub ciudad: String,
    pub genero: String,
    pub telefono: Option<String>,
    pub foto_url: Option<String>,
    pub tipo: String,
    pub estado: String,
    pub email_provider: Option<String>,
    pub metadata_json: Option<String>,
    pub creado_en: String,
    pub ultimo_uso: Option<String>,
}

impl Identidad {
    pub fn nombre_completo(&self) -> String {
        match &self.segundo_apellido {
            Some(s) => format!("{} {} {}", self.nombre, self.apellido, s),
            None => format!("{} {}", self.nombre, self.apellido),
        }
    }

    pub fn generar_email_gmail(&self) -> String {
        let base = format!(
            "{}.{}",
            self.nombre.to_lowercase(),
            self.apellido.to_lowercase()
        );
        // Quitar acentos y caracteres especiales
        let limpio: String = base
            .chars()
            .map(|c| match c {
                'á' | 'à' | 'ä' => 'a',
                'é' | 'è' | 'ë' => 'e',
                'í' | 'ì' | 'ï' => 'i',
                'ó' | 'ò' | 'ö' => 'o',
                'ú' | 'ù' | 'ü' => 'u',
                'ñ' => 'n',
                _ => c,
            })
            .collect();
        format!("{}@gmail.com", limpio)
    }

    pub fn generar_email_proton(&self) -> String {
        let base = format!(
            "{}.{}.{}",
            self.nombre.to_lowercase(),
            self.apellido.to_lowercase(),
            self.ciudad.to_lowercase()
        );
        let limpio: String = base
            .chars()
            .map(|c| match c {
                'á' | 'à' | 'ä' => 'a',
                'é' | 'è' | 'ë' => 'e',
                'í' | 'ì' | 'ï' => 'i',
                'ó' | 'ò' | 'ö' => 'o',
                'ú' | 'ù' | 'ü' => 'u',
                'ñ' => 'n',
                ' ' => '.',
                _ => c,
            })
            .collect();
        format!("{}@proton.me", limpio)
    }
}

// ─── Factory ────────────────────────────────────────────────────────────────
#[derive(Clone)]
pub struct IdentityFactory {
    nombres_masc: Vec<&'static str>,
    nombres_fem: Vec<&'static str>,
    apellidos: Vec<&'static str>,
    ciudades: Vec<(&'static str, &'static str)>, // (ciudad, país)
}

impl Default for IdentityFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl IdentityFactory {
    pub fn new() -> Self {
        Self {
            nombres_masc: Self::cargar_nombres_masculinos(),
            nombres_fem: Self::cargar_nombres_femeninos(),
            apellidos: Self::cargar_apellidos(),
            ciudades: Self::cargar_ciudades(),
        }
    }

    pub fn generar_identidad_base(&self) -> Identidad {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let genero = if rng.gen_bool(0.5) {
            "masculino"
        } else {
            "femenino"
        };
        let nombre = match genero {
            "masculino" => self.nombres_masc[rng.gen_range(0..self.nombres_masc.len())],
            _ => self.nombres_fem[rng.gen_range(0..self.nombres_fem.len())],
        };
        let apellido = self.apellidos[rng.gen_range(0..self.apellidos.len())];
        let segundo_apellido = if rng.gen_bool(0.6) {
            Some(self.apellidos[rng.gen_range(0..self.apellidos.len())])
        } else {
            None
        };
        let (ciudad, pais) = self.ciudades[rng.gen_range(0..self.ciudades.len())];
        let fecha_nac = self.generar_fecha_nacimiento(18, 65);
        let password = self.generar_password(16);

        Identidad {
            id: uuid::Uuid::new_v4().to_string(),
            nombre: nombre.to_string(),
            apellido: apellido.to_string(),
            segundo_apellido: segundo_apellido.map(|s| s.to_string()),
            email: String::new(), // Se asigna después según tipo
            password,
            recovery_email: None,
            fecha_nacimiento: fecha_nac,
            pais: pais.to_string(),
            ciudad: ciudad.to_string(),
            genero: genero.to_string(),
            telefono: None,
            foto_url: None,
            tipo: String::new(),
            estado: "Creada".to_string(),
            email_provider: None,
            metadata_json: None,
            creado_en: chrono::Utc::now().to_rfc3339(),
            ultimo_uso: None,
        }
    }

    pub fn generar_password(&self, length: usize) -> String {
        use rand::Rng;
        const CHARSET: &[u8] =
            b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*";
        let mut rng = rand::thread_rng();
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    fn generar_fecha_nacimiento(&self, min_edad: u32, max_edad: u32) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let edad = rng.gen_range(min_edad..=max_edad);
        let año = 2025 - edad;
        let mes = rng.gen_range(1..=12);
        let dia = rng.gen_range(1..=28);
        format!("{:04}-{:02}-{:02}", año, mes, dia)
    }

    fn cargar_nombres_masculinos() -> Vec<&'static str> {
        vec![
            "Alejandro",
            "Carlos",
            "David",
            "Eduardo",
            "Fernando",
            "Gabriel",
            "Héctor",
            "Ignacio",
            "Javier",
            "Kevin",
            "Luis",
            "Manuel",
            "Nicolás",
            "Óscar",
            "Pablo",
            "Ricardo",
            "Santiago",
            "Tomás",
            "Ulises",
            "Víctor",
            "Adrián",
            "Benjamín",
            "Cristóbal",
            "Diego",
            "Emilio",
            "Francisco",
            "Gonzalo",
            "Hugo",
            "Iván",
            "Jorge",
            "Leonardo",
            "Miguel",
            "Norberto",
            "Oliver",
            "Patricio",
            "Rafael",
            "Samuel",
            "Teodoro",
            "Uriel",
            "Valentín",
            "Andrés",
            "Bruno",
            "César",
            "Daniel",
            "Elías",
            "Fabián",
            "Gael",
            "Harold",
            "Isaac",
            "Julián",
            "Luciano",
            "Mateo",
            "Natán",
            "Orlando",
            "Pedro",
            "Raúl",
            "Sebastián",
            "Thiago",
            "Urbano",
            "Vicente",
            "Alan",
            "Brayan",
            "Cristian",
            "Damián",
            "Esteban",
            "Felipe",
            "Gerardo",
            "Hans",
            "Ismael",
            "Joel",
            "Kurt",
            "Lautaro",
            "Martín",
            "Néstor",
            "Omar",
            "Plutarco",
            "Ramiro",
            "Salvador",
            "Tadeo",
            "Ubaldo",
            "Valerio",
            "Wálter",
            "Xavier",
            "Yahir",
            "Zacarías",
        ]
    }

    fn cargar_nombres_femeninos() -> Vec<&'static str> {
        vec![
            "Alejandra",
            "Beatriz",
            "Carmen",
            "Daniela",
            "Elena",
            "Fernanda",
            "Gabriela",
            "Helena",
            "Inés",
            "Julia",
            "Karina",
            "Laura",
            "María",
            "Natalia",
            "Oriana",
            "Paula",
            "Rebeca",
            "Sofía",
            "Teresa",
            "Úrsula",
            "Valentina",
            "Adriana",
            "Berenice",
            "Carolina",
            "Diana",
            "Estefanía",
            "Florencia",
            "Gloria",
            "Heidy",
            "Iris",
            "Jimena",
            "Karla",
            "Liliana",
            "Mónica",
            "Noelia",
            "Olivia",
            "Patricia",
            "Rosa",
            "Silvia",
            "Tamara",
            "Uriel",
            "Verónica",
            "Abril",
            "Bárbara",
            "Camila",
            "Dulce",
            "Erika",
            "Fabiola",
            "Griselda",
            "Hilda",
            "Ilse",
            "Jazmín",
            "Leticia",
            "Marina",
            "Nadia",
            "Olga",
            "Pamela",
            "Raquel",
            "Sabrina",
            "Tatiana",
            "Úrsula",
            "Viviana",
            "Ximena",
            "Yolanda",
            "Zulma",
            "Amanda",
            "Brenda",
            "Claudia",
            "Denisse",
            "Emilia",
            "Fátima",
            "Guadalupe",
            "Heidi",
            "Ingrid",
            "Jessica",
            "Lorena",
            "Martha",
            "Nayeli",
            "Ofelia",
            "Perla",
            "Ruth",
            "Samantha",
            "Tania",
            "Valeria",
            "Wendy",
            "Xiomara",
            "Yamila",
            "Zoe",
        ]
    }

    fn cargar_apellidos() -> Vec<&'static str> {
        vec![
            "García",
            "Rodríguez",
            "Martínez",
            "López",
            "Hernández",
            "González",
            "Pérez",
            "Sánchez",
            "Ramírez",
            "Cruz",
            "Flores",
            "Morales",
            "Ortiz",
            "Castillo",
            "Reyes",
            "Gutiérrez",
            "Jiménez",
            "Mendoza",
            "Aguilar",
            "Ramos",
            "Vázquez",
            "Álvarez",
            "Romero",
            "Díaz",
            "Moreno",
            "Torres",
            "Chávez",
            "Rivera",
            "Herrera",
            "Peña",
            "Vargas",
            "Castro",
            "Guerrero",
            "Contreras",
            "Ortega",
            "Estrada",
            "Delgado",
            "Molina",
            "Rivas",
            "Campos",
            "Núñez",
            "Soto",
            "Silva",
            "Pacheco",
            "Acosta",
            "Medina",
            "Salazar",
            "Vega",
            "Cervantes",
            "Fuentes",
            "Cortés",
            "Rangel",
            "Zamora",
            "Cárdenas",
            "Valdez",
            "Gallegos",
            "Velázquez",
            "Márquez",
            "Sandoval",
            "Padilla",
            "Ríos",
            "Carrillo",
            "Solís",
            "Tovar",
            "Bautista",
            "Paredes",
            "Lara",
            "Escobar",
            "Bravo",
            "Quintero",
            "Bernal",
            "Mejía",
            "Navarro",
            "Rocha",
            "Ayala",
            "Ibarra",
            "Ponce",
            "Robles",
            "Alvarado",
            "Cordova",
            "Galván",
            "Trujillo",
            "Hidalgo",
            "Cisneros",
            "Zavala",
            "Barrios",
            "Sepúlveda",
            "Orozco",
            "Aragón",
            "Dueñas",
            "Sierra",
            "Palacios",
            "Tello",
            "Valencia",
            "Zúñiga",
            "Cuevas",
            "Santana",
            "Vera",
            "Coronel",
            "Espino",
            "Garmendia",
            "Portillo",
            "Mancilla",
            "Carranza",
            "Acevedo",
            "Peralta",
            "Segovia",
            "Barrientos",
            "Bueno",
            "Calvo",
            "Cano",
            "Carrasco",
            "Castañeda",
            "Escudero",
            "Espinoza",
            "Frías",
            "Gallardo",
            "Garrido",
            "Gil",
            "Gimeno",
            "Giménez",
            "Godoy",
            "Grande",
            "Guerra",
            "Haro",
            "Hidalgo",
            "Huerta",
            "Iglesias",
            "Infante",
            "Jaime",
            "Lago",
            "Leal",
            "León",
            "Llorente",
            "Lobo",
            "Lorenzo",
            "Lozano",
            "Madrid",
            "Marín",
            "Martín",
            "Mateo",
            "Méndez",
            "Miranda",
            "Montero",
            "Montes",
            "Mora",
            "Murillo",
            "Olivera",
            "Pascual",
            "Pastor",
            "Paz",
            "Perales",
            "Piña",
            "Plaza",
            "Polo",
            "Prado",
            "Puig",
            "Quesada",
            "Quintana",
            "Redondo",
            "Revilla",
            "Rey",
            "Robledo",
            "Roca",
            "Roig",
            "Rubio",
            "Sáenz",
            "Salamanca",
            "Sanz",
            "Sarabia",
            "Senderos",
        ]
    }

    fn cargar_ciudades() -> Vec<(&'static str, &'static str)> {
        vec![
            ("Ciudad de México", "México"),
            ("Buenos Aires", "Argentina"),
            ("Madrid", "España"),
            ("Bogotá", "Colombia"),
            ("Lima", "Perú"),
            ("Santiago", "Chile"),
            ("Caracas", "Venezuela"),
            ("Quito", "Ecuador"),
            ("La Paz", "Bolivia"),
            ("Montevideo", "Uruguay"),
            ("Asunción", "Paraguay"),
            ("San José", "Costa Rica"),
            ("San Salvador", "El Salvador"),
            ("Panamá", "Panamá"),
            ("Guatemala", "Guatemala"),
            ("Tegucigalpa", "Honduras"),
            ("Managua", "Nicaragua"),
            ("Santo Domingo", "República Dominicana"),
            ("Barcelona", "España"),
            ("Medellín", "Colombia"),
            ("Guadalajara", "México"),
            ("Monterrey", "México"),
            ("Valencia", "España"),
            ("Córdoba", "Argentina"),
            ("Rosario", "Argentina"),
            ("Cali", "Colombia"),
            ("Barranquilla", "Colombia"),
            ("Arequipa", "Perú"),
            ("Valparaíso", "Chile"),
            ("Maracaibo", "Venezuela"),
            ("Guayaquil", "Ecuador"),
            ("Santa Cruz", "Bolivia"),
            ("Puebla", "México"),
            ("Toluca", "México"),
            ("Sevilla", "España"),
            ("Málaga", "España"),
            ("Zaragoza", "España"),
            ("Murcia", "España"),
            ("Palma", "España"),
            ("Bilbao", "España"),
            ("Alicante", "España"),
            ("Mendoza", "Argentina"),
            ("Salta", "Argentina"),
            ("La Plata", "Argentina"),
            ("Tucumán", "Argentina"),
            ("Cartagena", "Colombia"),
            ("Bucaramanga", "Colombia"),
            ("Tijuana", "México"),
            ("Cancún", "México"),
            ("Mérida", "México"),
        ]
    }
}
