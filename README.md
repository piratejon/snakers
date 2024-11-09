# snake.rs
codename: ideal-lamp (this is the name GitHub suggested for this repo when it asked me to pick one)

## about
This is a snake game made in rust. I have only tried rust once before and I did not get very far. Now I have tried to make a snake game and it was basically easy and fun. This includes an alpine-based development container.

## files
* src/
    * snake.rs - game logic (library)
    * cli.rs - command line driver (binary)
* env/
    * build.sh - build the development container
    * run.sh - run the development container

# thinking

## animation
the game model is a grid. the snake's body occupies whole grid squares in the model. animation draws the snake between two adjacent grid squares gradually. each square of the snake's body can be thought of as having a leaving and entering square (or lagging and leading squares). during a frame, a snake body square collision-detects with the leading square. the first square in the snake's body is referred to as the head. the last square in the snake's body is referred to as the tail. the time it takes for the snake to advance one grid square is based on the game speed, G (i.e. 2 squares per second). each square in the snake's body is rendered incrementally further towards that soquare's leading square based on the frame rate, f (i.e. 30 Hz). therefore, the snake partially occupies both the lagging and leading square for f/G frames (i.e. 15 frames). f/G is expected to be greater than or equal to 1 (lower means multiple game ticks occur per frame which is probably undesirable). if the frame rate equals the game speed, then the snake's motion between squares appears discrete rather than smoothly animated. the frame rate may be dynamic (such as unlocked). the snake body part occuping square S1 in the model is drawn at incremental positions between lagging square S0 and leading square S1 between two game ticks based on the frame time between the two ticks. for the frame drawn at time t, with t0 being the time of the previous game tick and t1 being the time of the next tick (i.e. t0 <= t < t1, with t1-t0 being constant equal to G) then the proportion of the snake S1 square drawn in S1 is (t1 - t)/(t1 - t0) (which is referred to per-frame as the frame progress factor), and the proportion drawn in S0 is (1 - ((t1 - t)/(t1 - t0))). in the starting condition, t=t0 and the snake is drawn lagged entirely by 1. the next frame, some (at least infinitesimal) portion of the snake is drawn in its occupied square S1. the head's leading square is determined by a controller's input such as a player's command. considering the right-facing case, the head's leading square will either be at 90, 0, or 270 degrees from the head's lagging square (where 0 is absolute and pointing right). in the 0 degree case the snake is moving forward to the right. in the 90 and 270 degree cases the snake is turning up or down. each case's trajectory is fixed and the head must be projected along the trajectory with forward progress proportional to the frame progress factor.

## coordinate system
right (positive x) is 0 or 360 degrees or 0 or 2pi radians. up (positive y) is 90 degrees or pi/2 radians. left (negative x) is +/-180 degrees or +/-pi radians. down (negative y) is 270 degrees or -90 degrees or 3pi/2 or -pi/2 radians.
