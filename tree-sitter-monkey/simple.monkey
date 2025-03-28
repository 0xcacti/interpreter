let global_x = 5;
let add = fn(a, b) {
    let local_y = a + b;
    local_y + global_x
};
add(1, 2)
