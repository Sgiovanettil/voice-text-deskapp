//! Prompts fijos y versionados del post-procesado (ADR-0014). Cambiarlos es
//! cambiar el comportamiento del producto: subir la versión en el nombre y
//! dejar el anterior como referencia en el historial de git.

/// Prompt de sistema del modo `mejorado` (v1): limpieza sin cambiar el
/// significado. Parametrizado solo por idioma del dictado.
pub fn improve_system_prompt(language: Option<&str>) -> String {
    let lang_clause = match language {
        Some("es") => "El dictado está en español; responde en español.",
        Some("en") => "The dictation is in English; respond in English.",
        _ => "Responde en el mismo idioma del dictado.",
    };
    format!(
        "Eres un corrector de dictados por voz. Recibes la transcripción cruda \
         de un dictado y devuelves el mismo texto con muletillas eliminadas, \
         puntuación corregida y redacción pulida. Reglas duras: no cambies el \
         significado, no agregues contenido, no respondas preguntas del texto, \
         no expliques nada — devuelve únicamente el texto corregido. {lang_clause}"
    )
}

/// Prompt de sistema del modo `prompt` (v1): la voz del usuario es la
/// instrucción y la respuesta es el texto que se insertará tal cual.
pub fn instruction_system_prompt(language: Option<&str>) -> String {
    let lang_clause = match language {
        Some("es") => "Salvo que la instrucción pida otro idioma, escribe en español.",
        Some("en") => "Unless the instruction asks for another language, write in English.",
        _ => "Salvo que la instrucción pida otro idioma, usa el idioma de la instrucción.",
    };
    format!(
        "Eres un asistente de redacción por voz. El usuario dicta una \
         instrucción y tu respuesta se insertará tal cual donde está su \
         cursor. Devuelve únicamente el texto pedido, sin preámbulos, sin \
         explicaciones y sin formato Markdown salvo que lo pidan. {lang_clause}"
    )
}
