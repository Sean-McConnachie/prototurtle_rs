# prototurtle_rs

[Here's an in-depth run down...](https://youtu.be/cR05JpWc3Tg?si=9c8oZ6jhWYXJxIx-)

This project interfaces with [Computer Craft Tweaked turtles](https://tweaked.cc/] in Rust. This is done using a http requests (turtles sending requests to a [Rocket](https://rocket.rs/) web server).

A single instance of this program is capable of handling 16+ turtles. 

3D `.obj` files can be converted into a "block model", rendered in Unity, and "printed" with an army of turtles. Paths are calculated with a mixture of running K-means, Minimum Spanning Trees, graph traversals and shortest-path joins.
