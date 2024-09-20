#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum KeywordsType {
    None, // invalid keyword
    Definir, Constante, Nombre, // DONE
    Lanzar, // DONE
    Funcion, // DONE
    Si, Entonces, // DONE
    Hacer, Mientras, Para, // DONE
    Romper, Retornar, Continuar, // DONE
    Clase, Estatico, Publico // DONE
    , Extender, // TODO
    Intentar, Capturar, Finalmente, // DONE
    Exportar, Importar, Como, // DONE
}
const KEYWORDS: [KeywordsType; 24] = 
[
    KeywordsType::None,
    KeywordsType::Definir,
    KeywordsType::Constante,
    KeywordsType::Nombre,
    KeywordsType::Funcion,
    KeywordsType::Si,
    KeywordsType::Entonces,
    KeywordsType::Hacer,
    KeywordsType::Mientras,
    KeywordsType::Para,
    KeywordsType::Romper,
    KeywordsType::Retornar,
    KeywordsType::Continuar,
    KeywordsType::Clase,
    KeywordsType::Estatico,
    KeywordsType::Publico,
    KeywordsType::Extender,
    KeywordsType::Intentar,
    KeywordsType::Capturar,
    KeywordsType::Finalmente,
    KeywordsType::Exportar,
    KeywordsType::Importar,
    KeywordsType::Como,
    KeywordsType::Lanzar
];
impl KeywordsType {
    pub fn iter() -> [KeywordsType; 24] {
        KEYWORDS
    }
    pub fn as_str(&self) -> &str {
        match self {
            KeywordsType::None => "none",
            KeywordsType::Definir => "def",
            KeywordsType::Constante => "const",
            KeywordsType::Nombre => "nombre",
            KeywordsType::Funcion => "fn",
            KeywordsType::Si => "si",
            KeywordsType::Entonces => "ent",
            KeywordsType::Hacer => "hacer",
            KeywordsType::Mientras => "mien",
            KeywordsType::Para => "para",
            KeywordsType::Romper => "rom",
            KeywordsType::Retornar => "ret",
            KeywordsType::Continuar => "cont",
            KeywordsType::Clase => "clase",
            KeywordsType::Estatico => "est",
            KeywordsType::Publico => "pub",
            KeywordsType::Extender => "extiende",
            KeywordsType::Intentar => "intentar",
            KeywordsType::Capturar => "capturar",
            KeywordsType::Finalmente => "finalmente",
            KeywordsType::Exportar => "exportar",
            KeywordsType::Importar => "importar",
            KeywordsType::Como => "como",
            KeywordsType::Lanzar => "lanzar"
        }
    }
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}
