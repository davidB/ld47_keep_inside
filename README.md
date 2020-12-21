# ld47_keep_inside

My game for Ludum Dare 47 (see release [compo](https://github.com/davidB/ld47_keep_inside/releases/tag/compo))

And some evolutions, experimentations, wip since LD:

- upgrade to bevy 0.3, 0.4
- use lyon to draw entity (vs sprite) (via [Nilirad/bevy_prototype_lyon: Use Lyon in Bevy.](https://github.com/Nilirad/bevy_prototype_lyon))
- create a web version (via [mrk-its/bevy_webgl2: WebGL2 renderer plugin for Bevy game engine](https://github.com/mrk-its/bevy_webgl2))
- add effects (try easing via [bevy_extra/bevy_easings at master · mockersf/bevy_extra](https://github.com/mockersf/bevy_extra/tree/master/bevy_easings))
- use bazel to build
- host on itch.io

## to plublish web on itch.io

```sh
bazel run //itch.io:butler login
bazel build //itch.io:publish-web --verbose_failures --action_env=BUTLER_API_KEY=$(cat  $HOME/.config/itch/butler_creds)
```
