#[derive(PartialEq)]
pub enum KeywordsType {
    None, // invalid keyword
    Definir, Constante, Nombre, // done
    Funcion, // done
    Si, Entonces, // done
    Hacer, Mientras, Para, // done
    Romper, Retornar, Continuar, // done
    Clase, Estatico, Publico // done
    , Extender, Implementar, // todo
    Intentar, Capturar, Finalmente, // done
    Exportar, Importar, Como, // done
    Lanzar // todo
}
const KEYWORDS: [KeywordsType; 25] = 
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
    KeywordsType::Implementar,
    KeywordsType::Intentar,
    KeywordsType::Capturar,
    KeywordsType::Finalmente,
    KeywordsType::Exportar,
    KeywordsType::Importar,
    KeywordsType::Como,
    KeywordsType::Lanzar
];
impl KeywordsType {
    pub fn iter() -> [KeywordsType; 25] {
        KEYWORDS
    }
    pub fn as_str(&self) -> &str {
        match self {
            KeywordsType::None => "NONE",
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
            KeywordsType::Implementar => "impl",
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
impl Clone for KeywordsType {
    fn clone(&self) -> KeywordsType {
        match self {
            KeywordsType::None => KeywordsType::None,
            KeywordsType::Definir => KeywordsType::Definir,
            KeywordsType::Constante => KeywordsType::Constante,
            KeywordsType::Nombre => KeywordsType::Nombre,
            KeywordsType::Funcion => KeywordsType::Funcion,
            KeywordsType::Si => KeywordsType::Si,
            KeywordsType::Entonces => KeywordsType::Entonces,
            KeywordsType::Hacer => KeywordsType::Hacer,
            KeywordsType::Mientras => KeywordsType::Mientras,
            KeywordsType::Para => KeywordsType::Para,
            KeywordsType::Romper => KeywordsType::Romper,
            KeywordsType::Retornar => KeywordsType::Retornar,
            KeywordsType::Continuar => KeywordsType::Continuar,
            KeywordsType::Clase => KeywordsType::Clase,
            KeywordsType::Estatico => KeywordsType::Estatico,
            KeywordsType::Publico => KeywordsType::Publico,
            KeywordsType::Extender => KeywordsType::Extender,
            KeywordsType::Implementar => KeywordsType::Implementar,
            KeywordsType::Intentar => KeywordsType::Intentar,
            KeywordsType::Capturar => KeywordsType::Capturar,
            KeywordsType::Finalmente => KeywordsType::Finalmente,
            KeywordsType::Exportar => KeywordsType::Exportar,
            KeywordsType::Importar => KeywordsType::Importar,
            KeywordsType::Como => KeywordsType::Como,
            KeywordsType::Lanzar => KeywordsType::Lanzar
        }
    }
}
impl Copy for KeywordsType {}
