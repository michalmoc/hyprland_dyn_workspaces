Workspace system for Hyprland allowing to insert workspace between already existing.
Because of Hyprland limitation the system abuses workspace names.

I put those in my `hyprland.conf`:

```
bind = $mainMod, N, exec, hyprland_dyn_workspaces new next
bind = $mainMod Ctrl, Right, exec, hyprctl dispatch workspace $(hyprland_dyn_workspaces find next)
bind = $mainMod Ctrl, Left, exec, hyprctl dispatch workspace $(hyprland_dyn_workspaces find previous)
bind = $mainMod Ctrl SHIFT, Right, exec, hyprctl dispatch movetoworkspace $(hyprland_dyn_workspaces find next)
bind = $mainMod Ctrl SHIFT, Left, exec, hyprctl dispatch movetoworkspace $(hyprland_dyn_workspaces find previous)
```

because of the name dependency you must also change your workspaces display, so that they are
sorted by name. For example my wrapper around `hyprland-workspaces`:

```shell
#!/bin/sh

MONITOR=`hyprctl monitors -j | jq -r '.[0].name'`
hyprland-workspaces "$MONITOR" | jq -c --unbuffered -r -M 'sort_by(.name)'
```