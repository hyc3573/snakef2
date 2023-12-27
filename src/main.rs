use rand::Rng;
use rand::distributions::Uniform;
use sfml::graphics::*;
use sfml::system::*;
use sfml::window::*;
use sfml::SfBox;
use std::collections::VecDeque;
use std::iter::repeat;
use itertools::Itertools;

struct Snake {
    cells: VecDeque<Vector2i>,
    dir: Dir,
    last_dir: Dir,
}

impl Snake {
    fn new(pos: Vector2i, dir: Dir, n: usize) -> Self {
        return Self {
            cells: VecDeque::from_iter(
                repeat(pos)
                    .take(n)
                    .enumerate()
                    .map(|(i, x)| x - dir.value() * (i as i32)),
            ),
            dir,
            last_dir: dir,
        };
    }
}

struct Animation {
    from: Vector2i,
    to: Vector2i,
}

#[derive(Clone, Copy)]
enum RenderState {
    UPDATE,
    ANIMATE,
}

#[derive(Clone, Copy)]
enum Dir {
    UP,
    DOWN,
    RIGHT,
    LEFT,
}

impl Dir {
    fn value(&self) -> Vector2i {
        match self {
            Dir::UP => Vector2i::new(0, -1),
            Dir::DOWN => Vector2i::new(0, 1),
            Dir::LEFT => Vector2i::new(-1, 0),
            Dir::RIGHT => Vector2i::new(1, 0),
        }
    }
}

#[derive(Clone, Copy)]
enum Apple {
    None,
    At(Vector2i)
}

struct Game {
    win_size: Vector2i,
    game_size: Vector2i,
    cell_size: Vector2f,
    last_update: SfBox<Clock>,
    update_interval: f32,
    render_state: RenderState,
    snakes: Vec<Snake>,
    animations: Vec<Animation>,
    fps_clock: SfBox<Clock>,
    apple: Apple
}

impl Game {
    fn new() ->  Self {
        Self {
            win_size: Vector2i::new(800, 600),
            game_size: Vector2i::new(40, 30),
            cell_size: Vector2f::new(20.0, 20.0),
            last_update: Clock::start(),
            update_interval: 0.1,
            render_state: RenderState::UPDATE,
            snakes: vec![Snake::new(Vector2i::new(6, 1), Dir::RIGHT, 5),
                         Snake::new(Vector2i::new(34, 28), Dir::LEFT, 5)],
            animations: Vec::new(),
            fps_clock: Clock::start(),
            apple: Apple::None 
        }
    }
}

const NANUM: &[u8] = include_bytes!("NanumGothicCoding.ttf");

fn main() {
    let mut game = Game::new();

    let mut rng = rand::thread_rng();

    let mut font;
    unsafe {
        font = Font::from_memory(NANUM).unwrap();
    }
    let font = font;

    let mut window = RenderWindow::new(
        VideoMode::new(game.win_size.x as u32, game.win_size.y as u32, 32),
        "2P SNAKE",
        Style::CLOSE,
        &Default::default(),
    );

    loop {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => {
                    window.close();
                    return;
                }
                Event::KeyPressed{code, ..} => {
                    match code {
                        Key::Up => {
                            if !matches!(game.snakes[1].last_dir, Dir::DOWN) {
                                game.snakes[1].dir = Dir::UP;
                            }
                        }
                        Key::Down => {
                            if !matches!(game.snakes[1].last_dir, Dir::UP) {
                                game.snakes[1].dir = Dir::DOWN;
                            }
                        }
                        Key::Left => {
                            if !matches!(game.snakes[1].last_dir, Dir::RIGHT) {
                                game.snakes[1].dir = Dir::LEFT;
                            }
                        }
                        Key::Right => {
                            if !matches!(game.snakes[1].last_dir, Dir::LEFT) {
                                game.snakes[1].dir = Dir::RIGHT;
                            }
                        }

                        Key::W => {
                            if !matches!(game.snakes[0].last_dir, Dir::DOWN) {
                                game.snakes[0].dir = Dir::UP;
                            }
                        }
                        Key::S => {
                            if !matches!(game.snakes[0].last_dir, Dir::UP) {
                                game.snakes[0].dir = Dir::DOWN;
                            }
                        }
                        Key::A => {
                            if !matches!(game.snakes[0].last_dir, Dir::RIGHT) {
                                game.snakes[0].dir = Dir::LEFT;
                            }
                        }
                        Key::D => {
                            if !matches!(game.snakes[0].last_dir, Dir::LEFT) {
                                game.snakes[0].dir = Dir::RIGHT;
                            }
                        }
                        
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        let dt = game.fps_clock.restart().as_seconds();

        match game.render_state {
            RenderState::UPDATE => {
                game.last_update.restart();
                game.render_state = RenderState::ANIMATE;

                if matches!(game.apple, Apple::None) {
                    let mut n_positions = game.game_size.x * game.game_size.y;
                    let mut grid = vec![vec![false; game.game_size.y as usize]; game.game_size.x as usize];
                    for snake in &game.snakes {
                        for cell in &snake.cells {
                            grid[cell.x as usize][cell.y as usize] = true;
                            n_positions -= 1;
                        }
                    }

                    let dist = Uniform::new(0, n_positions);
                    let mut n = rng.sample(dist);
                    let mut pos = Vector2i::new(0, 0);

                    let mut index = 0;
                    for (i, column) in grid.iter().enumerate() {
                        for (j, elem) in column.iter().enumerate() {
                            if *elem && index <= n {
                                n += 1;
                            }
                            if n == index{
                                pos = Vector2i::new(i as i32, j as i32);
                            };
                            
                            index += 1;
                        }
                    }

                    game.apple = Apple::At(pos);
                }

                game.animations.clear();
                for snake in &mut game.snakes {
                    if snake.cells.is_empty() {
                        continue;
                    }
                    
                    let mut appled = false;
                    match game.apple {
                        Apple::At(pos) => {
                            if *snake.cells.front().unwrap() + snake.dir.value() == pos {
                                appled = true;
                                game.apple = Apple::None;
                            }
                        }
                        _ => {}
                    }

                    game.animations.push(
                        Animation {
                            from: *snake.cells.front().unwrap(),
                            to: *snake.cells.front().unwrap() + snake.dir.value()
                        }
                    );
                    
                    if snake.cells.len() >= 2 {
                        if appled {
                            game.animations.push(
                                Animation {
                                    from: *snake.cells.back().unwrap(),
                                    to: *snake.cells.back().unwrap()
                                }
                            )
                        } else {
                            game.animations.push(
                                Animation {
                                    from: *snake.cells.back().unwrap(),
                                    to: snake.cells[snake.cells.len()-2]
                                }
                            )
                        }
                    }

                    for cell in snake.cells.iter().take(snake.cells.len()-1) {
                        game.animations.push(
                            Animation {
                                from: *cell,
                                to: *cell
                            }
                        )
                    }

                    snake.cells.push_front(*snake.cells.front().unwrap() + snake.dir.value());
                    snake.last_dir = snake.dir;

                    if !appled {
                        snake.cells.pop_back();
                    }
                }

                let mut eliminate = Vec::<usize>::new();

                // Inter-collision
                for ((i, snake1), (j, snake2)) in game.snakes.iter().enumerate().combinations(2).map(|x| {(x[0], x[1])}) {
                    if snake1.cells.is_empty() {
                        continue;
                    }

                    let head1 = *snake1.cells.front().unwrap();
                    let head2 = *snake2.cells.front().unwrap();

                    for cell in snake2.cells.iter() {
                        if head1 == *cell {
                            // Snake1 eliminate
                            eliminate.push(i);
                            break;
                        }
                    }

                    for cell in snake1.cells.iter() {
                        if head2 == *cell {
                            eliminate.push(j);
                            break;
                        }
                    }
                }

                // Self-collision || Wall-collision
                for (i, snake) in game.snakes.iter().enumerate() {
                    if snake.cells.is_empty() {
                        continue;
                    }
                    
                    let head = *snake.cells.front().unwrap();

                    if head.x < 0 || head.x >= game.game_size.x || head.y < 0 || head.y >= game.game_size.y {
                            eliminate.push(i);
                        }

                    for cell in snake.cells.iter().skip(1) {
                        if head == *cell {
                            eliminate.push(i);
                        }
                    }
                }

                if !eliminate.is_empty() {
                    let mut message = "";
                    if eliminate.len() == 1 {
                        if eliminate[0] == 0 {
                            message = "Player 1 won"
                        } else {
                            message = "Player 2 won"
                        }
                    } else {
                        message = "Draw"
                    }

                    let mut exit = false;
                    loop {
                        while let Some(event) = window.poll_event() {
                            match event {
                                Event::Closed => {return}
                                Event::KeyPressed {..} => {
                                    exit = true;
                                }
                                _ => {}
                            }
                        } 

                        if exit {
                            game = Game::new();
                            
                            break;
                        }

                        window.clear(Color::WHITE);

                        let mut text = Text::new(message, &font, 24);
                        let rc = text.local_bounds();
                        text.set_origin(rc.size()/2.0);
                        text.set_position(game.win_size.as_other()/2.0_f32);
                        text.set_fill_color(Color::BLACK);
                        window.draw(&text);

                        let mut text = Text::new("Press Any Key...", &font, 24);
                        let rc = text.local_bounds();
                        text.set_origin(rc.size()/2.0);
                        text.set_position(game.win_size.as_other()/2.0_f32 + Vector2f::new(0.0, 50.0));
                        text.set_fill_color(Color::BLACK);
                        window.draw(&text);

                        window.display()
                    }

                    if exit {
                        continue;
                    }
                }
                
                window.clear(Color::WHITE);

                match game.apple {
                    Apple::At(pos) => {
                        let mut rect = RectangleShape::new();
                        rect.set_size::<Vector2f>(game.cell_size);
                        rect.set_position(game.cell_size.cwise_mul(pos.as_other()));
                        rect.set_fill_color(Color::RED);

                        window.draw(&rect);
                    }
                    _ => {}
                }

                for snake in &game.snakes {
                    for cell in &snake.cells {
                        let mut rect = RectangleShape::new();
                        rect.set_size::<Vector2f>(game.cell_size);
                        rect.set_position(game.cell_size.cwise_mul(cell.as_other()));
                        rect.set_fill_color(Color::GREEN);

                        window.draw(&rect);
                    }
                }

                window.display();
            }
            RenderState::ANIMATE => {
                let t = game.last_update.elapsed_time().as_seconds() / game.update_interval;
                
                if t > 1.0 {
                    game.render_state = RenderState::UPDATE;
                }

                
                window.clear(Color::WHITE);

                match game.apple {
                    Apple::At(pos) => {
                        let mut rect = RectangleShape::new();
                        rect.set_size::<Vector2f>(game.cell_size);
                        rect.set_position(game.cell_size.cwise_mul(pos.as_other()));
                        rect.set_fill_color(Color::RED);

                        window.draw(&rect);
                    }
                    _ => {}
                }

                for animation in &game.animations {
                    let mut rect = RectangleShape::new();
                    rect.set_size(game.cell_size);
                    rect.set_position(game.cell_size.cwise_mul(
                        animation.from.as_other()*(1.0-t) + animation.to.as_other()*t
                    ));
                    rect.set_fill_color(Color::GREEN);

                    window.draw(&rect);
                }

                window.display();
            }
        }
    }
}
