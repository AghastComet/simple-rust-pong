use glium::{implement_vertex, uniform};
use glium::Surface;

const PADDLE_HEIGHT: f32 = 0.02;
const PADDLE_WIDTH: f32 = 0.5;

const BALL_DIAMETER: f32 = 0.02;

const INITAL_BALL_SPEED: [f32; 2] = [0.006, -0.03];
const PADDLE_VELOCITY: f32 = 0.01;
const PLAYER_INITIAL_POSITION: [f32; 2] = [0.0, -0.9];
const BOT_INITIAL_POSITION: [f32; 2] = [0.0, 0.9];
const BALL_INITIAL_POSITION: [f32; 2] = [0.0, 0.0];

const BALL_SPEEDUP: f32 = 1.01;

const BALL_REFLECTION_MULTIPLYER: f32 = 0.05;

#[derive(Copy, Clone)]
struct Vertex{
    position: [f32; 2]
}

implement_vertex!(Vertex, position);

fn main() {
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let wb = glium::glutin::window::WindowBuilder::new().with_inner_size(glium::glutin::dpi::PhysicalSize::new(500, 500)).with_resizable(false).with_title("Pong");
    let cb = glium::glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let paddle_vertices = vec![
        Vertex{position: [-PADDLE_WIDTH/2., -PADDLE_HEIGHT/2.]},
        Vertex{position: [-PADDLE_WIDTH/2., PADDLE_HEIGHT/2.]},
        Vertex{position: [PADDLE_WIDTH/2., -PADDLE_HEIGHT/2.]},
        Vertex{position: [PADDLE_WIDTH/2., PADDLE_HEIGHT/2.]},
    ];
    let paddle_buffer = glium::VertexBuffer::new(&display, &paddle_vertices).unwrap();

    let ball_vertices = vec![
        Vertex{position: [-BALL_DIAMETER/2., -BALL_DIAMETER/2.]},
        Vertex{position: [-BALL_DIAMETER/2., BALL_DIAMETER/2.]},
        Vertex{position: [BALL_DIAMETER/2., -BALL_DIAMETER/2.]},
        Vertex{position: [BALL_DIAMETER/2., BALL_DIAMETER/2.]},
    ];
    let ball_buffer = glium::VertexBuffer::new(&display, &ball_vertices).unwrap();

    // Used by both buffers
    let indices = glium::index::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TriangleStrip,
        &[0,1,2,3_u8],
    ).unwrap();


    let vertex_shader_source = r#"
        #version 140

        in vec2 position;
        uniform vec2 offset_position;

        void main(){
            vec2 final_position = position.xy + offset_position;
            gl_Position = vec4(final_position, 0.0, 1.0);
        }

    "#;

    let fragment_shader_source = r#"
        #version 140

        out vec4 color;

        void main(){
            color = vec4(1.0, 1.0, 1.0, 1.0);
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_source, fragment_shader_source, None).unwrap();

    let mut player: [f32; 2] = PLAYER_INITIAL_POSITION;
    let mut bot: [f32; 2] = BOT_INITIAL_POSITION;
    let mut ball: [f32; 2] = BALL_INITIAL_POSITION;
    let mut ball_velocity: [f32; 2] = INITAL_BALL_SPEED;




    let mut left_pressed = false;
    let mut right_pressed = false;

    // let mut previous_frame_time = std::time::Instant::now();
    let mut next_frame_time = std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);

    event_loop.run(move |ev, _, control_flow|{
        match ev{
            glium::glutin::event::Event::WindowEvent{event, .. } => match event {
                glium::glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glium::glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                _ => (),
            },
            glium::glutin::event::Event::DeviceEvent{event, ..} => match event{
                glium::glutin::event::DeviceEvent::Key(key) => {
                    if key.virtual_keycode == Some(glium::glutin::event::VirtualKeyCode::Left){
                        left_pressed = key.state == glium::glutin::event::ElementState::Pressed;
                    } else if key.virtual_keycode == Some(glium::glutin::event::VirtualKeyCode::Right){
                        right_pressed = key.state == glium::glutin::event::ElementState::Pressed;
                    }
                }
                _ => (),
            }
            _ => (),
        }

        let current_frame_time = std::time::Instant::now();
        if current_frame_time >= next_frame_time{
            let time_delta = (current_frame_time - next_frame_time).as_nanos() as f32 /1000000000. + 1.;

            next_frame_time = std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
            *control_flow = glium::glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

            ball[0] += ball_velocity[0] * time_delta;
            ball[1] += ball_velocity[1] * time_delta;

            if ball[0] - 0.05 > bot[0]{
                bot[0] = ball[0].min(bot[0] + PADDLE_VELOCITY * time_delta);
            } else if ball[0] + 0.05 < bot[0]{
                bot[0] = ball[0].max(bot[0] - PADDLE_VELOCITY * time_delta);
            }

            if left_pressed {
                player[0] -= PADDLE_VELOCITY * time_delta;
            } else if right_pressed {
                player[0] += PADDLE_VELOCITY * time_delta;
            }

            if ball_velocity[1] < 0. && (ball[1] - player[1]).abs() <= (BALL_DIAMETER+PADDLE_HEIGHT)/2. && (ball[0] - player[0]).abs() <= (BALL_DIAMETER+PADDLE_WIDTH)/2.{
                // Ball collided with player
                ball_velocity[0] = (ball[0]-player[0])/PADDLE_WIDTH * BALL_REFLECTION_MULTIPLYER;
                ball_velocity[1] *= -BALL_SPEEDUP;
            } else if ball_velocity[1] > 0. && (ball[1] - bot[1]).abs() <= (BALL_DIAMETER+PADDLE_HEIGHT)/2. && (ball[0] - bot[0]).abs() <= (BALL_DIAMETER+PADDLE_WIDTH)/2.{
                ball_velocity[0] = (ball[0]-bot[0])/PADDLE_WIDTH * BALL_REFLECTION_MULTIPLYER;
                ball_velocity[1] *= -BALL_SPEEDUP;
                // Ball collided with bot
            } else if ball[0] > 0.99 || ball[0] < -0.99 {
                // Ball collided with wall
                ball_velocity[0] = -ball_velocity[0];
            } else if ball[1].abs() > 1.{
                // Ball hit score area
                // TODO: count scores
                player = PLAYER_INITIAL_POSITION;
                bot = BOT_INITIAL_POSITION;
                ball = BALL_INITIAL_POSITION;
                ball_velocity = INITAL_BALL_SPEED;
            }

            let mut target = display.draw();

            target.clear_color(0., 0., 0., 1.);

            target.draw(&paddle_buffer, &indices, &program, &uniform!{offset_position: player}, &Default::default()).unwrap();
            target.draw(&paddle_buffer, &indices, &program, &uniform!{offset_position: bot}, &Default::default()).unwrap();
            target.draw(&ball_buffer, &indices, &program, &uniform!{offset_position: ball}, &Default::default()).unwrap();

            target.finish().unwrap();
        }

    });
}
