# Podlet demo using VHS (https://github.com/charmbracelet/vhs)

Output demo.gif

Set Width 1650
Set Height 1100
Set LineHeight 1.5

# Setup
Hide
Type "cargo build --release && alias podlet=target/release/podlet && clear"
Enter
Sleep 500ms
Show

Sleep 500ms
Type "podlet -h"
Sleep 1s
Enter
Sleep 5s

Hide
Type "clear"
Enter
Show
Sleep 500ms

Type "podlet podman -h"
Sleep 1s
Enter
Sleep 5s

Hide
Type "clear"
Enter
Show
Sleep 500ms

Type "podlet \"
Enter
Type "  podman run \"
Enter
Type "  -p 8000:80 \"
Enter
Type "  -p 8443:443 \"
Enter
Type "  -v ./Caddyfile:/etc/caddy/Caddyfile \"
Enter
Type "  -v caddy_data:/data \"
Enter
Type "  docker.io/library/caddy:latest"
Sleep 3s
Enter
Sleep 5s

Hide
Type "clear"
Enter
Show
Sleep 500ms

Type "podlet --file . --install \"
Enter
Type "  podman run \"
Enter
Type "  --restart always \"
Enter
Type "  -p 8000:80 \"
Enter
Type "  -p 8443:443 \"
Enter
Type "  -v ./Caddyfile:/etc/caddy/Caddyfile \"
Enter
Type "  -v caddy_data:/data \"
Enter
Type "  docker.io/library/caddy:latest"
Sleep 3s
Enter
Sleep 3s
Type "cat caddy.container"
Enter
Sleep 8s

# Cleanup
Hide
Type "unalias podlet; rm caddy.container"
Enter
