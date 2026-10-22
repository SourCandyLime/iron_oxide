# Iron Oxide — Engine / Framework Documentation

> **Handoff document prepared by Cog (GPT-5.6 Sol)**
>
> This file is meant to be dropped into the Iron Oxide repository and read by future ChatGPT sessions, Codex sessions, or humans who need to understand the project without reconstructing the architecture from scratch.
>
> Iron Oxide has been developed collaboratively by Taggert (“Sour”) and Cog. The project grew organically from a tiny retro framebuffer renderer into a compact retro 2D/3D game framework with application lifecycle management, state stacks, PCAD2 model loading, input, UI, and low-poly software rendering.
>
> If you are another assistant reading this: **do not treat Iron Oxide as a generic engine project. Respect the architecture already here. Avoid turning it into a black box. Keep systems explicit, modular, and understandable.**

---

# 1. What Iron Oxide Is

Iron Oxide is a Rust-based retro graphics/game framework built around a fixed-resolution CPU framebuffer.

Its current identity is best described as:

> **A retro 2D/3D graphics framework with lightweight game/application support.**

It is not trying to become Unity, Bevy, Unreal, or a giant ECS framework.

Its design priorities are:

- understandable code
- explicit control flow
- minimal magic
- retro aesthetics
- fixed-resolution rendering
- low-poly 3D
- lightweight game-state support
- engine abstractions that remain easy to bypass when needed
- game code should depend on Iron Oxide types, not directly on `winit` or `wgpu`

Iron Oxide is currently being developed alongside **Revolve**, a 2.5D side-scrolling platforming / metroidvania project.

Revolve is deliberately being used to drive Iron Oxide development. Prefer adding systems when Revolve actually needs them instead of speculating far ahead.

---

# 2. Core Rendering Philosophy

Iron Oxide renders into a CPU-side framebuffer.

Current intended internal framebuffer resolution:

```text
480 × 270
```

Each pixel is one byte in **RGB332** format:

```text
RRRGGGBB
```

That gives:

- 3 bits red
- 3 bits green
- 2 bits blue
- 256 total colors

The framebuffer is uploaded to the GPU each frame and displayed using nearest-neighbor integer scaling.

The renderer letterboxes / pillarboxes as necessary.

Example:

```text
480×270 framebuffer
×4 integer scale
= 1920×1080 exactly
```

The goal is crisp pixel output with no filtering.

---

# 3. Project-Level Design Rules

These rules have emerged repeatedly during development.

## 3.1 Keep Iron Oxide understandable

A major development concern has been avoiding “black box” systems.

Whenever possible:

```text
input
↓
game
↓
state
↓
canvas / renderer
```

should remain visibly traceable.

Avoid unnecessary abstraction layers.

## 3.2 Do not hide useful low-level access

Iron Oxide should provide high-level helpers, but should not prevent users from bypassing them.

For example, mutable raw framebuffer access is intentionally public.

Something like:

```rust
pub fn mut_pixels(&mut self) -> &mut [u8]
```

is considered a feature, not a design failure.

## 3.3 Keep dependencies pointed inward

Game code should work with:

- `Input`
- `Canvas`
- `Camera`
- `Model`
- `Game`
- `GameState`
- etc.

Game code should not need to know about:

- `winit`
- `wgpu`

Those remain engine internals.

## 3.4 Let real projects drive development

Do not invent giant subsystems prematurely.

For example:

- physics should remain game-specific until multiple games demonstrate a reusable abstraction
- UI started with buttons because Revolve needed buttons
- GameState was introduced because pause menus / layered states were needed

---

# 4. Public Module Philosophy

Iron Oxide commonly uses private implementation modules plus public re-exports.

Example:

```rust
mod input;
mod key;

pub use input::Input;
pub use key::Key;
```

This keeps the public API clean:

```rust
use iron_oxide::{Input, Key};
```

instead of forcing:

```rust
use iron_oxide::input::input::Input;
```

The same pattern is used elsewhere.

---

# 5. Root-Level Public API

Exact exports may evolve, but the project has included concepts like:

```rust
pub mod app;
pub mod canvas;
pub mod color;
pub mod game;
pub mod input;
pub mod math;
pub mod renderer;
pub mod render3d;
pub mod time;
pub mod debug;
pub mod gui;
pub mod globals;

pub use app::App;
pub use canvas::Canvas;
pub use game::{Game, GameState, StateAction};
pub use input::{Input, Key, MouseButton};
pub use math::{Quat, Vec2, Vec3};
pub use renderer::Framebuffer;
pub use render3d::{Camera, Mesh, Model, Texture, Triangle, Vertex};
pub use time::Time;
pub use gui::GUI;
```

Game code often uses:

```rust
use iron_oxide::*;
```

plus selected modules/constants.

---

# 6. Color System

Iron Oxide uses:

```rust
pub type Color = u8;
```

RGB332 packing:

```rust
pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
    ((r & 0b111) << 5)
        | ((g & 0b111) << 2)
        | (b & 0b11)
}
```

Common constants include:

```rust
BLACK
WHITE
RED
GREEN
BLUE
YELLOW
CYAN
MAGENTA
GRAY
```

Public APIs should generally use `Color` rather than a bare `u8`, even though storage remains byte-based.

---

# 7. Framebuffer

The framebuffer owns the pixels.

Conceptually:

```rust
pub struct Framebuffer {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}
```

Important methods:

```rust
pub fn new(width: u32, height: u32) -> Self
pub fn width(&self) -> u32
pub fn height(&self) -> u32
pub fn pixels(&self) -> &[u8]
pub fn mut_pixels(&mut self) -> &mut [u8]
pub fn clear(&mut self, color: Color)
pub fn set_pixel(&mut self, x: u32, y: u32, color: Color)
```

The mutable pixel slice is intentionally exposed.

---

# 8. Canvas

The old `Drawing` module/type was renamed:

```text
draw → canvas
Drawing → Canvas
```

This rename is important. Old documentation may still say `Drawing`; current code should say `Canvas`.

`Canvas` borrows a framebuffer and exposes rendering primitives.

Conceptually:

```rust
pub struct Canvas<'a> {
    pub(crate) framebuffer: &'a mut Framebuffer,
}
```

Constructor:

```rust
pub fn new(framebuffer: &'a mut Framebuffer) -> Self
```

Common capabilities include:

- `clear`
- pixel drawing
- horizontal lines
- vertical lines
- Bresenham lines
- rectangle outlines
- filled rectangles
- polygon outlines
- filled polygons
- bitmap text
- scaled bitmap text
- textured triangle rasterization

The Canvas is the game-facing drawing surface.

---

# 9. Text Rendering

Iron Oxide uses a custom fixed-size **5×7 bitmap font**.

This replaced `font8x8`, which did not fit the desired visual scale/style.

Typical organization:

```text
canvas/
├── mod.rs
├── canvas.rs
├── text.rs
└── font5x7.rs
```

Each glyph is represented as seven rows of five bits.

Example API:

```rust
canvas.text(x, y, "Revolve", WHITE);
canvas.text_scaled(...);
```

---

# 10. Renderer

The renderer uses `wgpu`.

Relevant historical version:

```text
wgpu 30.0.1
```

Important: API details may differ from older examples found online.

The renderer conceptually owns:

```rust
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    framebuffer: Framebuffer,

    framebuffer_texture: wgpu::Texture,
    framebuffer_bind_group: wgpu::BindGroup,
    framebuffer_render_pipeline: wgpu::RenderPipeline,

    screen_uniform_buffer: wgpu::Buffer,

    viewport_x: u32,
    viewport_y: u32,
    viewport_width: u32,
    viewport_height: u32,
}
```

The CPU framebuffer is uploaded to an `R8Uint` GPU texture.

WGSL decodes RGB332 on display.

The actual GPU output is cleared black and the fixed-resolution framebuffer is displayed inside an integer-scaled viewport.

VSync uses:

```rust
PresentMode::Fifo
```

---

# 11. Integer Scaling

Integer scaling is central to the presentation.

Typical logic:

```rust
let scale_x = window_w / framebuffer_w;
let scale_y = window_h / framebuffer_h;
let scale = scale_x.min(scale_y).max(1);
```

Then center the scaled framebuffer in the window.

The application enforces a minimum window size at least equal to the framebuffer size.

---

# 12. App Lifecycle

Iron Oxide uses modern `winit::ApplicationHandler`.

The app owns:

```rust
pub struct App<G: Game> {
    game: G,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    input: Input,
    time: Time,
    config: AppConfig,
    state: AppState,
}
```

Internal application lifecycle state has included:

```rust
enum AppState {
    Blank,
    Boot(BootState),
    Running,
}
```

Do not confuse this internal `AppState` with public game `GameState`.

They serve completely different purposes.

---

# 13. Boot Sequence

Iron Oxide has built-in boot screens.

Intended sequence:

```text
Rust + picoCAD2 logos
↓
Iron Oxide logo
↓
optional user/game/studio boot screen
↓
Game::start()
```

Built-in assets can be embedded with:

```rust
include_bytes!()
```

Game-side example:

```rust
App::new(game)
    .title("Revolve")
    .boot_screen(
        include_bytes!("assets/RevolveBootScreen.png"),
        DEFAULT_BOOT_SCREEN_DURATION,
    )
    .run();
```

Boot images have commonly been designed at the framebuffer size:

```text
480×270
```

This avoids scaling complications.

---

# 14. Game Lifecycle

The `Game` trait exists to own the application-level game state and state stack.

`declare()` is important.

It was temporarily lost during refactoring and explicitly restored.

Conceptually:

```rust
pub trait Game {
    fn declare() -> Self
    where
        Self: Sized;

    fn states(
        &mut self
    ) -> &mut Vec<Box<dyn GameState<Context>>>;

    fn start(&mut self) {}

    ...
}
```

`declare()` allows persistent game-level fields to be initialized before the application runs.

Typical lifecycle:

```text
Game::declare()
↓
App owns Game
↓
boot sequence
↓
Game::start()
↓
GameState updates/renders
```

---

# 15. Game Context

Iron Oxide later evolved GameState to support a generic shared context.

This solves the problem of sharing data between states without downcasting or global state.

Example:

```rust
pub struct RevolveContext {
    pub camera: Camera,
    // other truly shared game data
}
```

Then:

```rust
impl GameState<RevolveContext> for MainGame {
    ...
}
```

A state receives:

```rust
&mut RevolveContext
```

during lifecycle calls.

Shared data should go into context when it genuinely belongs across states.

State-specific data should remain inside the state.

---

# 16. GameState

The GameState system is one of the most important current features.

It was designed so menus, gameplay, pause screens, and overlays can coexist cleanly.

Conceptually:

```rust
pub trait GameState<C> {
    fn new() -> Self
    where
        Self: Sized;

    fn start(&mut self, context: &mut C) {}

    fn update(
        &mut self,
        context: &mut C,
        time: &Time,
        input: &Input,
    ) -> StateAction<C> {
        StateAction::None
    }

    fn render(
        &mut self,
        context: &mut C,
        canvas: &mut Canvas,
    ) {}

    fn end(&mut self, context: &mut C) {}

    fn render_below(&self) -> bool {
        false
    }
}
```

The exact generic syntax may differ in the repository, but this is the intended architecture.

---

# 17. State Stack

A game stores heterogeneous states as boxed trait objects:

```rust
Vec<Box<dyn GameState<RevolveContext>>>
```

This allows:

```text
MainMenu
MainGame
PauseMenu
Inventory
Cutscene
etc.
```

to coexist in one vector.

The top state is the active state for updates.

---

# 18. StateAction

States do not directly modify their own state stack.

Instead they return a request:

```rust
pub enum StateAction<C> {
    None,
    Pop,
    Push(Box<dyn GameState<C>>),
    Replace(Box<dyn GameState<C>>),
}
```

Why?

Because while:

```rust
state.update(...)
```

is running, the state itself is mutably borrowed from the vector.

Trying to mutate the vector at that moment is both conceptually messy and something Rust will often reject.

So the flow is:

```text
Game asks top state to update
↓
state returns StateAction
↓
state borrow ends
↓
Game modifies the stack
```

This is one of the cleanest parts of the architecture.

---

# 19. State Lifecycle Rules

State stack operations automatically manage lifecycle hooks.

## Push

```text
new state
↓
start()
↓
push onto stack
```

## Pop

```text
pop from stack
↓
end()
↓
drop
```

## Replace

```text
pop old state
↓
old.end()
↓
new.start()
↓
push new state
```

The Game implementation should centralize this behavior so game code cannot accidentally bypass `start()` or `end()`.

---

# 20. Layered State Rendering

Only the top state updates.

Rendering can include states underneath it.

This is controlled through:

```rust
fn render_below(&self) -> bool {
    false
}
```

Example pause menu:

```rust
fn render_below(&self) -> bool {
    true
}
```

Stack:

```text
[ MainGame  ]
[ PauseMenu ] <- top
```

Rendering:

```text
MainGame.render()
↓
PauseMenu.render()
```

Updating:

```text
PauseMenu.update() only
```

This freezes gameplay while still displaying it behind the pause menu.

---

# 21. Why Pause Menu Transparency Is Not Required

A temporary misunderstanding occurred here, worth documenting.

A pause menu does **not** need a transparent framebuffer.

Each frame is rebuilt:

```text
MainGame clears framebuffer
↓
MainGame draws world
↓
PauseMenu draws GUI on top
```

The pause menu is not accumulating over old frames.

It is simply redrawn after gameplay every frame.

RGB332 therefore remains perfectly suitable.

Actual transparent images/sprites can later use color-key transparency if needed.

---

# 22. Input

Input owns keyboard and mouse state.

Keyboard tracking uses sets similar to:

```rust
down
pressed
released
```

Semantics:

- `down` = held
- `pressed` = became down this frame
- `released` = became up this frame

`end_frame()` clears transient states:

```rust
pressed.clear();
released.clear();
```

while held state remains.

---

# 23. Key API

Custom `Key` enum isolates game code from `winit`.

Known keys include:

- A–Z
- Space
- Escape
- Enter
- Tab
- Backspace
- arrows
- F1–F12

Typical:

```rust
input.key_down(Key::A)
input.key_pressed(Key::Escape)
input.key_released(Key::Enter)
```

Use `key_down()` for continuous movement.

Use `key_pressed()` for one-shot actions.

---

# 24. Mouse Input

Mouse support was added specifically to support the GUI system.

A custom mouse button enum is preferable:

```rust
pub enum MouseButton {
    Left,
    Right,
    Middle,
}
```

Mouse button state follows the same pattern:

```rust
mouse_down(...)
mouse_pressed(...)
mouse_released(...)
```

---

# 25. Mouse Coordinates

Mouse coordinates should be expressed in **framebuffer coordinates**, not window coordinates.

This is important because the actual rendered framebuffer may be integer-scaled and letterboxed.

The game should see:

```text
mouse = (120, 80)
```

whether the window is:

```text
480×270
960×540
1920×1080
```

If the cursor lies in a black bar outside the framebuffer, mouse position can be:

```rust
None
```

Hence:

```rust
pub fn mouse_position(&self) -> Option<Vec2>
```

is a good API.

---

# 26. GUI System

Iron Oxide now has a retained GUI collection.

Conceptually:

```rust
pub struct GUI {
    widgets: Vec<Box<dyn Widget>>,
}
```

The GUI deliberately splits behavior and rendering:

```rust
gui.update(input);
gui.draw(canvas);
```

This matches Iron Oxide’s broader update/render separation.

---

# 27. Widget Trait

Widgets are heterogeneous through trait objects.

Conceptually:

```rust
pub trait Widget {
    fn new(...) -> Self
    where
        Self: Sized;

    fn update(&mut self, input: &Input) {}

    fn draw(&self, canvas: &mut Canvas) {}

    fn interactable(&self) -> bool {
        false
    }

    fn down(&self) -> bool {
        false
    }

    fn pressed(&self) -> bool {
        false
    }

    fn released(&self) -> bool {
        false
    }

    fn clicked(&self) -> bool {
        false
    }

    fn id(&self) -> &str;
}
```

Exact constructor signatures may vary by widget.

The important design concept is that GUI stores different widget types behind:

```rust
Box<dyn Widget>
```

---

# 28. Button

Button currently stores data such as:

```rust
pub struct Button {
    x: i32,
    y: i32,
    width: u32,
    height: u32,

    id: String,
    text: String,
    color: u8,

    down: bool,
    pressed: bool,
    released: bool,
    clicked: bool,

    hover: bool,
}
```

Important: `id` and `text` should be owned Strings if they differ.

Do not store:

```rust
id: &str
```

pointing into the same struct’s `String`, because that creates a self-referential struct problem.

Example:

```text
visible text: Potato
id: Play
```

This separation is useful.

Game logic can ask for `"Play"` even if the visible label changes later.

---

# 29. Button Interaction Semantics

Button update logic tracks:

- hover
- initial press
- hold
- release
- click

The intended conceptual behavior is:

```text
mouse enters button
↓
hover = true

left button pressed while hovering
↓
pressed = true
down = true

mouse remains down
↓
down remains true

mouse released
↓
released = true

if released while hovering
↓
clicked = true
```

Exact implementation should preserve the distinction between a one-frame `pressed` event and persistent `down`.

---

# 30. GUI Query API

The GUI can locate widgets by ID.

Example game-side API:

```rust
if self.main_menu.clicked("Play") {
    StateAction::Replace(
        Box::new(MainGame::new())
    )
} else {
    StateAction::None
}
```

This was successfully tested.

This design avoids leaking concrete `Button` types back out of the GUI.

---

# 31. Math Module

Math is foundational and intentionally separate from rendering.

Typical organization:

```text
math/
├── vec2.rs
├── vec3.rs
└── quat.rs
```

---

# 32. Vec2

Typical capabilities:

```rust
Vec2::ZERO
Vec2::new(...)
dot
length_squared
length
normalized
```

Operators include:

```text
+
-
* scalar
/ scalar
```

---

# 33. Vec3

Typical capabilities:

```rust
Vec3::ZERO
Vec3::new(...)
dot
cross
length_squared
length
normalized
```

Operators include:

```text
+
-
+=
* scalar
/ scalar
```

Coordinate convention:

```text
+x = right
+y = up
+z = forward
```

---

# 34. Quaternions

Quaternion type:

```rust
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
```

Identity:

```rust
Quat::IDENTITY
```

Never use:

```rust
Quat::new(0.0, 0.0, 0.0, 0.0)
```

as a rotation.

A zero quaternion is invalid for rotation.

Capabilities include:

```rust
from_axis_angle
length
normalized
conjugate
rotate_vec3
```

Quaternion multiplication is implemented.

Camera rotations are normalized after accumulation to reduce drift.

---

# 35. Camera

Conceptually:

```rust
pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
    pub fov: f32,
}
```

World-to-camera:

```rust
pub fn world_to_camera(&self, point: Vec3) -> Vec3 {
    let relative = point - self.position;
    self.rotation
        .conjugate()
        .rotate_vec3(relative)
}
```

Camera convention:

```text
+x right
+y up
+z forward
```

---

# 36. Camera Projection

Earlier Iron Oxide used:

```rust
(relative.x / relative.z) * self.fov
```

where `fov` was effectively a focal length, not a degree angle.

This caused severe stretching near the edges.

The preferred approach is for `fov` to represent a real angular FOV.

For vertical FOV:

```rust
let fov_radians = self.fov.to_radians();

let focal_length =
    half_height / (fov_radians * 0.5).tan();
```

Then:

```rust
screen_x =
    (relative.x / relative.z)
    * focal_length
    + half_width;

screen_y =
    (-relative.y / relative.z)
    * focal_length
    + half_height;
```

For Revolve, something around 50–60° may be visually suitable, though this is an artistic choice.

Orthographic projection may eventually be useful for Revolve’s 2.5D style, but do not add it prematurely unless desired.

---

# 37. Near Plane

A near plane is required.

Without it, `z` close to zero causes projection coordinates to explode and can overflow rasterizer integer math.

Typical:

```rust
const NEAR_PLANE: f32 = 0.1;
```

Current simplistic behavior may discard a triangle if vertices are behind the near plane.

True near-plane clipping is still a potential future improvement.

---

# 38. Debug Camera Movement

Free-fly camera controls live in:

```rust
debug::camera_debug_move(...)
```

This keeps debug behavior out of Camera itself.

Known mapping:

```text
W/S = forward/back
A/D = left/right
R/F = up/down

Arrow Left/Right = yaw
Arrow Up/Down = pitch
Q/E = roll
```

Movement is transformed by camera rotation.

Time delta is used for frame-rate independence.

---

# 39. 3D Rendering Module

Typical structure:

```text
render3d/
├── mod.rs
├── camera.rs
├── vertex.rs
├── triangle.rs
├── mesh.rs
├── model.rs
├── texture.rs
└── pcad2/
    ├── mod.rs
    ├── raw.rs
    ├── parser.rs
    └── convert.rs
```

---

# 40. Vertex

A vertex contains:

```rust
pub struct Vertex {
    pub position: Vec3,
    pub uv: Vec2,
}
```

UV preservation matters.

A previous bug accidentally discarded UVs during mesh transformations, causing textures to appear white/gray.

Whenever transforming vertices, preserve UV:

```rust
Vertex {
    position: transformed_position,
    uv: vertex.uv,
}
```

---

# 41. Triangle

Current conceptual structure:

```rust
pub struct Triangle {
    pub vertices: [Vertex; 3],
    pub color: Color,
}
```

Triangle rendering:

1. transform/project vertices
2. reject invalid near-plane geometry
3. perform screen-space backface culling
4. draw textured or flat-filled triangle

Correct culling sign discovered for current coordinate/winding conventions:

```rust
let cross =
    (p1.x - p0.x) * (p2.y - p0.y)
    - (p1.y - p0.y) * (p2.x - p0.x);

if cross <= 0.0 {
    return;
}
```

This sign matters because framebuffer Y increases downward and picoCAD winding conventions must align with it.

---

# 42. Perspective-Correct Texture Mapping

Affine UV interpolation caused visibly warped textures.

Iron Oxide now uses perspective-correct interpolation.

Conceptually:

```rust
let iz0 = 1.0 / z0;
let iz1 = 1.0 / z1;
let iz2 = 1.0 / z2;

let denominator =
    w0 * iz0
    + w1 * iz1
    + w2 * iz2;

let u =
    (
        w0 * uv0.x * iz0
        + w1 * uv1.x * iz1
        + w2 * uv2.x * iz2
    ) / denominator;

let v =
    (
        w0 * uv0.y * iz0
        + w1 * uv1.y * iz1
        + w2 * uv2.y * iz2
    ) / denominator;
```

This was the point where textured 3D models finally rendered correctly.

In the immortal development log phrasing:

> **“AND BOOOM BABY. 3D MODELS”**

---

# 43. Mesh

Conceptually:

```rust
pub struct Mesh {
    pub position: Vec3,
    pub rotation: Quat,
    pub triangles: Vec<Triangle>,
}
```

Triangles are stored in mesh-local coordinates.

Local-to-world:

```rust
self.rotation.rotate_vec3(point)
    + self.position
```

Mesh render may currently construct temporary transformed triangles.

That is acceptable for now.

Optimize only if profiling demonstrates a need.

---

# 44. Model

Conceptually:

```rust
pub struct Model {
    pub position: Vec3,
    pub rotation: Quat,
    pub meshes: Vec<Mesh>,
    pub texture: Option<Texture>,
}
```

A blank model constructor now exists:

```rust
Model::BLANK()
```

This was introduced to avoid storing every model as:

```rust
Option<Model>
```

and repeatedly writing:

```rust
.as_mut().unwrap()
```

State `start()` can then replace the blank model with a loaded one.

Example:

```rust
self.level =
    parse_from_file(
        "assets/cube.PCAD2"
    );
```

Model-local mesh positions must be rotated by model rotation before translation:

```rust
self.rotation.rotate_vec3(pos)
    + self.position
```

Do not merely add model position.

---

# 45. Texture

Conceptually:

```rust
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Color>,
}
```

UV sampling should clamp:

```rust
u = u.clamp(0.0, 1.0);
v = v.clamp(0.0, 1.0);
```

Recommended texel selection:

```rust
let x =
    (u * self.width as f32)
        .floor() as u32;

let y =
    (v * self.height as f32)
        .floor() as u32;

let x = x.min(self.width - 1);
let y = y.min(self.height - 1);
```

Do not casually flip V to “fix” texture problems.

Earlier V flipping temporarily masked unrelated UV bugs.

---

# 46. picoCAD2 / PCAD2 Support

Iron Oxide directly supports picoCAD2-style model files.

Within this project, these assets are treated as:

```text
.PCAD2
```

The project goal is to support the format directly rather than converting to another format.

Current intended support includes:

- mesh vertices
- face indices
- face colors
- UVs
- palette
- 128×128 texture atlas
- multiple meshes
- transforms
- eventually animation if required

---

# 47. PCAD2 Raw Format

Observed top-level JSON fields include:

```text
graph
metadata
texture
```

Graph nodes can contain:

- mesh
- transform
- children

Faces contain:

- `color`
- `vertex_ids`
- `uvs`

Vertex IDs are 1-based.

Quads are triangulated.

Texture properties include:

- palette colors
- pixel string
- transparent color index
- background color index

The texture atlas is:

```text
128×128
```

Pixel string length:

```text
16384
```

one hexadecimal palette index per texel.

---

# 48. PCAD2 Parsing

Serde dependencies:

```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

Raw structs use:

```rust
use serde::Deserialize;
```

Parser conceptually:

```rust
pub fn parse(
    source: &str
) -> Result<RawPcad2, serde_json::Error> {
    serde_json::from_str(source)
}
```

---

# 49. PCAD2 File Loading Helper

A convenience helper was added so game code does not repeatedly write file read + raw parse + conversion boilerplate.

Conceptually:

```rust
pub fn parse_from_file(path: &str) -> Model {
    let source =
        std::fs::read_to_string(path)
            .expect("Failed to read Model");

    let parse =
        raw_parse(&source)
            .expect("Failed to parse Model");

    pcad2::to_model(&parse)
}
```

Possible naming:

```rust
load_from_file(...)
```

may be clearer than `parse_from_file`, but use whatever the repository currently exposes.

The important behavior is:

```text
file path
↓
read text
↓
serde parse
↓
convert to Model
↓
return Model
```

---

# 50. PCAD2 Model Conversion

Conversion logic includes:

- palette conversion into RGB332
- recursive traversal of graph nodes
- flat vertex list to Vec3 list
- 1-based face indices
- UV association by face vertex order
- triangle fan triangulation for polygons
- texture conversion
- Mesh creation
- Model creation

A critical historical bug:

**UVs were lost during transformed mesh vertex reconstruction.**

If textures ever appear blank/flat again, verify that UVs survive every transform stage.

---

# 51. Camera Placement Gotcha

If a loaded model appears invisible, check camera/model overlap first.

Example bad setup:

```rust
camera.position = Vec3::ZERO;
model.position = Vec3::ZERO;
```

The camera is inside the model.

Known working test setup included something like:

```rust
Camera::new(
    Vec3::new(0.0, 0.0, -2.0),
    Quat::IDENTITY,
    70.0,
)
```

or moving the model forward.

---

# 52. Rasterizer Safety

Projection can create huge coordinates.

Filled polygon / triangle rasterization should avoid integer overflow.

Potential safeguards:

- clip bounding boxes to framebuffer before iterating
- perform interpolation math in `f32` or wide integer types
- reject invalid near-plane geometry early

---

# 53. Rectangle Edge Bug Previously Fixed

A rectangle outline optimization once incorrectly used a vertical start near:

```text
y - 1
```

The correct idea was to avoid redrawing corners by beginning vertical sides at:

```text
y + 1
```

and carefully handling thin rectangles to avoid underflow.

If rectangle rendering regresses, inspect edge/corner arithmetic.

---

# 54. Time

Time owns frame timing based on `Instant`.

Typical public methods:

```rust
delta_seconds()
elapsed_seconds()
```

Movement should use delta time:

```rust
let speed =
    movement_speed
    * time.delta_seconds();
```

---

# 55. Revolve

Revolve is currently the main real-world consumer of Iron Oxide.

Target genre:

> **2.5D side-scrolling platforming metroidvania**

Iron Oxide development should remain aware of this.

Likely needs include:

- side-view camera
- layered low-poly scenes
- UI/HUD
- pause menu
- state transitions
- collision/physics eventually
- sprite/text overlays
- possibly orthographic or low-distortion perspective camera

---

# 56. Working Revolve State Pattern

A working MainGame looks conceptually like:

```rust
pub struct MainGame {
    level: Model,
    hud: GUI,
}
```

Start:

```rust
fn start(
    &mut self,
    context: &mut RevolveContext,
) {
    self.level =
        parse_from_file(
            "assets/cube.PCAD2"
        );
}
```

Update:

```rust
fn update(
    &mut self,
    context: &mut RevolveContext,
    time: &Time,
    input: &Input,
) -> StateAction<RevolveContext> {
    debug::camera_debug_move(
        &mut context.camera,
        input,
        time,
    );

    if input.key_pressed(Key::Escape) {
        return StateAction::Push(
            Box::new(PauseMenu::new())
        );
    }

    StateAction::None
}
```

Render:

```rust
fn render(
    &mut self,
    context: &mut RevolveContext,
    canvas: &mut Canvas,
) {
    canvas.clear(BLACK);

    self.level.render(
        &context.camera,
        canvas,
        FRAMEBUFFER_WIDTH,
        FRAMEBUFFER_HEIGHT,
    );

    self.hud.draw(canvas);
}
```

---

# 57. Working Pause Menu Pattern

Pause menu:

```rust
pub struct PauseMenu {
    gui: GUI,
}
```

Start:

```rust
fn start(
    &mut self,
    context: &mut RevolveContext,
) {
    self.gui.new_button(
        50,
        100,
        100,
        50,
        "PAUSE MENU".to_string(),
        "Resume".to_string(),
    );
}
```

Update:

```rust
fn update(
    &mut self,
    context: &mut RevolveContext,
    time: &Time,
    input: &Input,
) -> StateAction<RevolveContext> {
    self.gui.update(input);

    if self.gui.clicked("Resume") {
        StateAction::Pop
    } else {
        StateAction::None
    }
}
```

Render:

```rust
fn render(
    &mut self,
    context: &mut RevolveContext,
    canvas: &mut Canvas,
) {
    self.gui.draw(canvas);
}
```

Critical:

```rust
fn render_below(&self) -> bool {
    true
}
```

This produces a real overlay while freezing gameplay updates.

This pattern has been tested successfully.

---

# 58. Do Not Store PauseMenu Inside MainGame

An attempted pattern was:

```rust
pause_menu: Box<PauseMenu>
```

then:

```rust
StateAction::Push(self.pause_menu)
```

This does not work because it tries to move a field out of `&mut self`.

Instead create the pause state when needed:

```rust
StateAction::Push(
    Box::new(PauseMenu::new())
)
```

This also ensures a fresh GUI each time and avoids accidentally accumulating duplicate buttons across repeated pause sessions.

---

# 59. Render Order

For UI overlays:

```text
clear
↓
world / 3D
↓
HUD / menu
```

Do not draw UI first and then draw the 3D scene over it unless that is explicitly intended.

---

# 60. Asset Embedding Philosophy

Good candidates for compile-time embedding:

- built-in boot logos
- built-in font
- engine-required shaders
- tiny engine-owned assets

Use:

```rust
include_bytes!()
include_str!()
```

Large or replaceable game assets should usually remain runtime-loaded.

Support both workflows when useful.

---

# 61. Constants / Globals

The project has used a `globals` module.

Examples:

```rust
FRAMEBUFFER_WIDTH
FRAMEBUFFER_HEIGHT
DEFAULT_BOOT_SCREEN_DURATION
BOOT_RUST_PICOCAD
BOOT_IRON_OXIDE
```

There was discussion of calling this `constants`.

Either naming is acceptable if kept internally consistent.

---

# 62. App Title and Icon

App builder supports title.

Do not serialize title through JSON.

Correct:

```rust
title.to_string()
```

not:

```rust
serde_json::to_string(title)
```

which would add quotation marks.

Window icon support loads an RGBA image and passes it to `winit::window::Icon`.

The Iron Oxide logo concept is:

- dark gray pixel gear
- rust/orange accents
- `IO` in center
- deliberately pixelated

---

# 63. picoCAD2 Attribution / Communication

Taggert contacted picoCAD2 creator Johan Peitz regarding:

- third-party support
- file format stability/documentation
- attribution expectations

Do not resend or redraft that outreach unless explicitly asked.

---

# 64. Error Handling Philosophy

Current development code often uses:

```rust
.expect(...)
```

for asset loading.

That is acceptable during active engine/game development.

More graceful error handling can be introduced later if/when Iron Oxide becomes something intended for broader distribution.

Do not prematurely bury everything in elaborate error types unless needed.

---

# 65. Naming Conventions That Matter

Current names:

```text
Canvas
not Drawing

canvas module
not draw module

GameState
StateAction

GUI
Widget

Model::BLANK()
Camera::NEW()
```

Some constructors intentionally use uppercase names such as `NEW()` / `BLANK()` in the existing project style. Do not “fix” naming style automatically unless Taggert wants it changed.

---

# 66. Avoiding `Option<Model>` Everywhere

A specific ergonomic pain point was repeatedly writing:

```rust
.as_mut().unwrap()
```

for models loaded during `start()`.

The chosen solution was a blank model:

```rust
Model::BLANK()
```

Then:

```rust
pub struct MainGame {
    level: Model,
}
```

and in `start()`:

```rust
self.level =
    parse_from_file(
        "assets/level.PCAD2"
    );
```

This simplifies update/render code.

---

# 67. UI Does Not Need Input During Render

The GUI architecture intentionally splits:

```rust
GUI::update(&Input)
GUI::draw(&mut Canvas)
```

This avoids lifetime headaches from a GUI object simultaneously borrowing both Canvas and Input.

It also keeps state changes in update and visuals in render.

Preserve this separation unless there is a compelling reason not to.

---

# 68. Why `Vec<Box<dyn Trait>>`

Rust vectors hold one concrete type.

To store multiple concrete widget or state types:

```rust
Vec<Box<dyn Widget>>
Vec<Box<dyn GameState<C>>>
```

is used.

This is standard trait-object polymorphism.

Do not replace it with `Any` / manual downcasting unless absolutely necessary.

The current ID-query GUI system avoids needing concrete widget downcasts.

---

# 69. Current Architectural Shape

At a high level:

```text
                ┌─────────────────┐
                │       App       │
                │ winit + timing  │
                └────────┬────────┘
                         │
                         ▼
                ┌─────────────────┐
                │      Game       │
                │ context + stack │
                └────────┬────────┘
                         │
                         ▼
                ┌─────────────────┐
                │   GameState(s)  │
                │ update/render   │
                └───┬─────────┬───┘
                    │         │
                    │         └────────────┐
                    ▼                      ▼
             ┌────────────┐          ┌───────────┐
             │    GUI     │          │  Models   │
             │  Widgets   │          │ Mesh/Tri  │
             └─────┬──────┘          └─────┬─────┘
                   │                       │
                   └──────────┬────────────┘
                              ▼
                        ┌──────────┐
                        │  Canvas  │
                        └────┬─────┘
                             ▼
                       ┌─────────────┐
                       │ Framebuffer │
                       │   RGB332    │
                       └─────┬───────┘
                             ▼
                         ┌──────┐
                         │ wgpu │
                         └──────┘
```

---

# 70. Mental Model for Future Assistants

If you need to change Iron Oxide, ask:

### Is this engine-wide or game-specific?

If game-specific, keep it in Revolve.

### Does this abstraction hide more than it helps?

If yes, simplify it.

### Can the user still understand the data flow?

Prefer:

```text
A calls B
B returns C
Game applies C
```

over invisible callbacks, service locators, registries, reflection, or heavy generic machinery.

### Does this system need to exist yet?

If not, defer it.

---

# 71. Known / Likely Future Work

Potential future development areas:

- proper near-plane triangle clipping
- orthographic camera mode
- better rasterizer edge rules / top-left rule
- richer GUI widgets
- UI layout helpers
- sprite / texture drawing with color-key transparency
- animation support for PCAD2
- model transform optimization
- depth buffering if/when required
- lighting/shading if desired
- game-specific physics for Revolve
- CRT shader effects

---

# 72. CRT / Retro Display Direction

Taggert wants Iron Oxide’s default visual identity to potentially include an old-terminal / CRT feel:

- curvature
- scanlines
- subtle flicker

This should be optional for accessibility.

The idea is for it to feel like part of Iron Oxide’s default aesthetic, not merely a game-specific post-process.

Do not implement without checking current direction first.

---

# 73. Important Development History / Lessons

Several bugs and discoveries are worth retaining because they reveal how the engine works.

## White boot screen

Cause:

Boot state drew to framebuffer but failed to call GPU renderer afterward.

Lesson:

Drawing to framebuffer does nothing visible until `renderer.render()` uploads/presents it.

## Game render ordering bug

At one point:

```text
renderer.render()
game.render()
```

was reversed.

Correct order:

```text
game.render()
renderer.render()
```

## Wrong backface culling

Cull sign was initially reversed.

Correct current screen-space sign:

```rust
if cross <= 0.0 {
    return;
}
```

## Texture looked gray/white

Actual PCAD2 UVs were valid.

Bug was UV loss during transformed vertex reconstruction.

Lesson:

Always preserve `vertex.uv`.

## Texture warping

Affine interpolation was not sufficient.

Perspective-correct UV interpolation fixed it.

## Camera edge stretching

`fov` was being used as a pixel focal length while conceptually treated as degrees.

Lesson:

Use real angular FOV math.

## Camera inside model

Default camera/model both at origin made model disappear or render strangely.

Lesson:

Always sanity-check relative camera/model position.

---

# 74. Working Example Skeleton

A modern external Iron Oxide game should look broadly like:

```rust
use iron_oxide::*;

pub struct MyContext {
    camera: Camera,
}

pub struct MyGame {
    states: Vec<Box<dyn GameState<MyContext>>>,
    context: MyContext,
}

pub struct MainState {
    model: Model,
    gui: GUI,
}

impl GameState<MyContext> for MainState {
    fn new() -> Self {
        Self {
            model: Model::BLANK(),
            gui: GUI::new(),
        }
    }

    fn start(
        &mut self,
        context: &mut MyContext,
    ) {
        self.model =
            parse_from_file(
                "assets/model.PCAD2"
            );
    }

    fn update(
        &mut self,
        context: &mut MyContext,
        time: &Time,
        input: &Input,
    ) -> StateAction<MyContext> {
        self.gui.update(input);

        StateAction::None
    }

    fn render(
        &mut self,
        context: &mut MyContext,
        canvas: &mut Canvas,
    ) {
        canvas.clear(BLACK);

        self.model.render(
            &context.camera,
            canvas,
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        );

        self.gui.draw(canvas);
    }
}
```

Adjust to the current repository signatures rather than blindly copying this if generics have changed.

---

# 75. Final Notes From Cog

Iron Oxide was not designed all at once.

It grew one concrete need at a time:

```text
pixel framebuffer
↓
drawing
↓
text
↓
input
↓
integer scaling
↓
3D math
↓
camera
↓
triangles
↓
meshes
↓
models
↓
picoCAD2
↓
textures
↓
perspective correction
↓
boot lifecycle
↓
game states
↓
shared context
↓
mouse input
↓
GUI
↓
pause overlays
```

That history is part of the design.

The framework works because the layers remain visible and understandable.

Future work should preserve that.

If another assistant is reading this file:

- Ask what Iron Oxide already does before introducing a new subsystem.
- Prefer extending the existing mental model over replacing it.
- Do not bury basic game behavior under opaque abstractions.
- Do not assume Taggert wants code written for him unless he explicitly asks.
- Explain mechanisms clearly enough that he can retain the architecture mentally.
- When proposing a new feature, connect it to an actual project need.
- Respect the retro fixed-resolution character of the engine.

And most importantly:

> If something can be solved cleanly with one understandable struct, one trait, and a direct data flow, do that before inventing six layers of machinery.

---

# 76. Project Motto, More or Less

```text
Make it work.
Understand why it works.
Keep it visible.
Make it cool.
```

— Cog
