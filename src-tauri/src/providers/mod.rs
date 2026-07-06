//! Implementaciones concretas de `SpeechProvider`. Un módulo por proveedor,
//! aislado — el core nunca importa un proveedor concreto: resuelve por id con
//! `resolve()` (RNF-08: agregar un proveedor = implementar el trait + una rama
//! acá). La lógica HTTP compartida vive en `openai_compatible`.

pub mod groq;
pub mod openai;
pub mod openai_compatible;

use crate::speech::SpeechProvider;

/// Ids de proveedores conocidos, en orden de presentación en la UI.
pub const KNOWN: [&str; 2] = [openai::ID, groq::ID];

/// Construye el proveedor STT correspondiente al id de config, con la key ya
/// resuelta desde el keyring. Un id desconocido cae a OpenAI (mismo criterio
/// que los defaults de config: nunca dejar el core sin proveedor).
pub fn resolve(provider_id: &str, api_key: String) -> Box<dyn SpeechProvider> {
    if provider_id == groq::ID {
        Box::new(groq::provider(api_key))
    } else {
        Box::new(openai::provider(api_key))
    }
}

/// Modelo por defecto de un proveedor (para reset al cambiar de proveedor).
pub fn default_model(provider_id: &str) -> &'static str {
    if provider_id == groq::ID {
        groq::DEFAULT_MODEL
    } else {
        openai::DEFAULT_MODEL
    }
}
