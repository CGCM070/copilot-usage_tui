# copilot-usage_cli

GitHub Copilot usage tracker CLI - Una herramienta para visualizar el uso de GitHub Copilot Pro desde la terminal.

![Screenshot](screenshot.png)

## Características

- **Dashboard interactivo** con barras de progreso y estadísticas por modelo
- **Múltiples temas**: dark, light, dracula, nord, monokai, gruvbox
- **Integración con Waybar** para mostrar el uso en la barra de Hyprland
- **Cacheo inteligente** con TTL configurable (por defecto 5 minutos)
- **Setup interactivo** para la primera configuración

## Instalación

### Compilar desde el código fuente

```bash
# Clonar el repositorio
git clone https://github.com/tu-usuario/copilot-usage_cli.git
cd copilot-usage_cli

# Compilar en modo release
cargo build --release

# Instalar en ~/.local/bin
cp target/release/copilot-usage_cli ~/.local/bin/

# O instalar globalmente
cargo install --path .
```

## Configuración inicial

La primera vez que ejecutes el programa, se iniciará el setup interactivo:

```bash
copilot-usage_cli
```

### Crear el Personal Access Token

1. Ve a: https://github.com/settings/personal-access-tokens/new
2. **Token name**: Cualquier nombre (ej: "Copilot Usage CLI")
3. **Resource owner**: Selecciona tu cuenta
4. **Expiration**: Elige la duración que prefieras
5. **Repository access**: "Public repositories" es suficiente
6. **Account permissions** (IMPORTANTE):
   - Busca **"Plan"** (NO "Copilot Requests")
   - Selecciona **"Read-only"**
7. Genera el token y cópialo

**⚠️ NOTA**: El permiso necesario es **"Plan"**, no "Copilot Requests". Son diferentes permisos.

### Seleccionar tema

Después de introducir el token, elige tu tema preferido:
- `dark` (por defecto)
- `light`
- `dracula`
- `nord`
- `monokai`
- `gruvbox`

La configuración se guarda en: `~/.config/copilot-usage_cli/config.toml`

## Uso

### Ver dashboard
```bash
copilot-usage_cli
```

### Forzar actualización (ignorar caché)
```bash
copilot-usage_cli --refresh
```

### Ver estado del caché
```bash
copilot-usage_cli --cache-status
```

### Cambiar tema temporalmente
```bash
copilot-usage_cli --theme nord
```

### Reconfigurar (cambiar token)
```bash
copilot-usage_cli reconfigure
# o
copilot-usage_cli reset
```

Si hay un error de autenticación, el programa te preguntará automáticamente si quieres reconfigurar.

### Ver configuración
```bash
copilot-usage_cli config
```

### Resetear configuración
```bash
copilot-usage_cli reset
```

## Integración con Waybar

Para mostrar el uso de Copilot en Waybar, añade esto a tu configuración de Waybar (`~/.config/waybar/config`):

```json
"custom/copilot": {
  "exec": "copilot-usage_cli --waybar",
  "interval": 300,
  "return-type": "json",
  "format": " {}",
  "tooltip": true,
  "class": "copilot-usage"
}
```

Y añade los estilos CSS en `~/.config/waybar/style.css`:

```css
#custom-copilot {
  padding: 0 10px;
  margin: 0 5px;
  color: #a6e3a1;
}

#custom-copilot.copilot-critical {
  color: #f38ba8;
}

#custom-copilot.copilot-warning {
  color: #fab387;
}

#custom-copilot.copilot-normal {
  color: #f9e2af;
}

#custom-copilot.copilot-low {
  color: #a6e3a1;
}
```

## Estructura de archivos

```
~/.config/copilot-usage_cli/
└── config.toml          # Configuración

~/.cache/copilot-usage_cli/
└── usage.json            # Cache de datos
```

## Configuración manual

Ejemplo de `~/.config/copilot-usage_cli/config.toml`:

```toml
token = "ghp_tu_token_aqui"
theme = "dark"
cache_ttl_minutes = 5
waybar_format = "{percentage}%"
```

## Atajos de teclado en el Dashboard

- `q` o `Esc`: Salir del dashboard

## Dependencias

- Rust 1.70+ (para compilar)
- GitHub Personal Access Token con permiso `Plan (Read)`

## License

MIT