extern crate shaderc;

#[derive(Debug)]
pub enum ShaderCompilationError {
    InvalidShaderExtension,
    IoError(std::io::Error),
    CompilerInitializationFailed,
    CompilationFailed(shaderc::Error),
}

impl std::fmt::Display for ShaderCompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderCompilationError::InvalidShaderExtension => write!(f, "Invalid shader extension"),
            ShaderCompilationError::IoError(e) => write!(f, "I/O error ({})", e),
            ShaderCompilationError::CompilerInitializationFailed => {
                write!(f, "Compiler initialization failed")
            }
            ShaderCompilationError::CompilationFailed(e) => write!(f, "Compilation failed ({})", e),
        }
    }
}

impl std::error::Error for ShaderCompilationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ShaderCompilationError::IoError(e) => Some(e),
            ShaderCompilationError::CompilationFailed(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ShaderCompilationError {
    fn from(e: std::io::Error) -> Self {
        ShaderCompilationError::IoError(e)
    }
}

impl From<shaderc::Error> for ShaderCompilationError {
    fn from(e: shaderc::Error) -> Self {
        ShaderCompilationError::CompilationFailed(e)
    }
}

pub fn compile_shader_into_spirv(
    in_path: std::path::PathBuf,
    out_path: std::path::PathBuf,
) -> Result<(), ShaderCompilationError> {
    let shader_kind = match in_path
        .extension()
        .unwrap_or(std::ffi::OsStr::new(""))
        .to_str()
        .unwrap_or("")
    {
        "vert" => shaderc::ShaderKind::Vertex,
        "frag" => shaderc::ShaderKind::Fragment,
        "geom" => shaderc::ShaderKind::Geometry,
        "comp" => shaderc::ShaderKind::Compute,
        _ => return Err(ShaderCompilationError::InvalidShaderExtension),
    };

    let source = std::fs::read_to_string(&in_path)?;

    let mut compiler = match shaderc::Compiler::new() {
        Some(compiler) => compiler,
        None => return Err(ShaderCompilationError::CompilerInitializationFailed),
    };

    let compiled_file = compiler.compile_into_spirv(
        &source,
        shader_kind,
        in_path.to_str().unwrap_or("unnamed_shader"),
        "main",
        None,
    )?;

    std::fs::write(&out_path, compiled_file.as_binary_u8())?;
    Ok(())
}

pub fn compile_shaders_into_spirv(
    in_dir: std::path::PathBuf,
    out_dir: std::path::PathBuf,
) -> Result<(), ShaderCompilationError> {
    std::fs::create_dir_all(out_dir.clone())?;
    for in_subdir in std::fs::read_dir(in_dir)? {
        let in_subdir = in_subdir?;
        if in_subdir.file_type()?.is_file() {
            let in_path = in_subdir.path();
            let mut out_path = std::path::PathBuf::from(out_dir.clone());
            out_path.push(format!(
                "{}.spv",
                in_path
                    .file_name()
                    .unwrap_or(std::ffi::OsStr::new(""))
                    .to_str()
                    .unwrap_or("unnamed"),
            ));
            compile_shader_into_spirv(in_path, out_path)?;
        }
    }
    Ok(())
}
