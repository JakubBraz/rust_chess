console.log("script start!");

let canvasHTML = document.getElementById("chess-board");

let icons = {
    "p": "♟︎",
    "r": "♜",
    "b": "♝",
    "n": "♞",
    "q": "♛",
    "k": "♚"
};

function draw_board(color) {
    let context = canvasHTML.getContext("2d");

    let lightColor = '#ddb180';
    let darkColor = '#7c330c';
    let useDark = color == "white";
    let w = context.canvas.width / 8;
    let h = context.canvas.height / 8;

    console.log(w, h);

    let boardStr = "rnbqkbnr\npppppppp\n        \n        \n        \n        \nPPPPPPPP\nRNBQKBNR\n";

    for (let col = 0; col < 8; col++) {
        useDark = !useDark;
        for (let row= 0; row < 8; row++) {
            context.fillStyle = useDark ? darkColor : lightColor;
            context.fillRect(row * w, col * h, w, h);
            context.font = "100px Arial";
            context.fillStyle = "black";
            // context.fillText("♙", row * w, col * h);
            context.fillText("♟︎", row * w, (col + 0.85) * h);
            console.log("♟︎", row * w, (col + 0.85) * h);
            useDark = !useDark;
        }
    }

    // context.font = "100px Arial";
    // context.fillStyle = "black";
    // // context.fillText("♙", row * w, col * h);
    // context.fillText("♟︎", 0, 400);
}

canvasHTML.addEventListener("mouseup", event => {
    console.log("jest event", event);
    console.log(
        event.clientX - canvasHTML.getBoundingClientRect().left,
        event.clientY - canvasHTML.getBoundingClientRect().top);
})

draw_board("white");

const socket = new WebSocket("ws://127.0.0.1:9977");
socket.addEventListener("open", (event) => {
    console.log("socket open event: ", event);
    let m = {"msg_type": "Join", "room_id": 123, "make_move": [[1,1], [2,2]]}
    socket.send(JSON.stringify(m));
    console.log("message sent");
});

socket.addEventListener("message", event => {
    console.log("message received:", event.data);
});
