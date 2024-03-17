console.log("script start!");

let canvasHTML = document.getElementById("chess-board");
let context = canvasHTML.getContext("2d");
let SQUARE_WIDTH = context.canvas.width / 8;
let SQUARE_HEIGHT = context.canvas.height / 8;
console.log(SQUARE_WIDTH, SQUARE_HEIGHT);

let PLAYER_COLOR = "white";

let icons = {
    "p": "♟︎",
    "r": "♜",
    "b": "♝",
    "n": "♞",
    "q": "♛",
    "k": "♚",
    " ": " "
};


function parse_board(color, boardStr) {
    console.log(boardStr);
    let board = boardStr.split("\n");
    console.log(board);
    return board;
}

function click_to_coords(color, x, y) {
    let row = Math.floor(y / SQUARE_HEIGHT);
    let col = Math.floor(x / SQUARE_WIDTH);
    if(color === "white") {
        row = Math.floor((context.canvas.height - y) / SQUARE_HEIGHT);
    }
    row = Math.min(7, row);
    row = Math.max(0, row);
    col = Math.min(7, col);
    col = Math.max(0, col);
    return [row, col];
}

function draw_board(color) {
    let lightColor = '#ddb180';
    let darkColor = '#7c330c';
    // let useDark = color == "white";

    let board = parse_board(color, "RNBQKBNR\nPPPPPPPP\n        \n        \n        \n        \npppppppp\nrnbqkbnr");

    let rowIndices = color == "white" ? [7, 6, 5, 4, 3, 2, 1, 0] : [0, 1, 2, 3, 4, 5, 6, 7];
    let colIndices = color == "white" ? [0, 1, 2, 3, 4, 5, 6, 7] : [7, 6, 5, 4, 3, 2, 1, 0];

    for (let row_on_screen= 0; row_on_screen < 8; row_on_screen++) {
        let row = rowIndices[row_on_screen];
        for (let col_on_screen = 0; col_on_screen < 8; col_on_screen++) {
            let col = colIndices[col_on_screen];
            context.fillStyle = row % 2 != col % 2 ? lightColor : darkColor;
            context.fillRect(col_on_screen * SQUARE_WIDTH, row_on_screen * SQUARE_HEIGHT, SQUARE_WIDTH, SQUARE_HEIGHT);
            context.font = "90px Arial";
            let element = board[row][col];
            let piece = icons[element.toLowerCase()];
            context.fillStyle = "prnbqk".includes(element) ? "black" : "white";
            context.fillText(piece, (col_on_screen + 0.05) * SQUARE_WIDTH, (row_on_screen + 0.85) * SQUARE_HEIGHT);
        }
    }
}

canvasHTML.addEventListener("mouseup", event => {
    console.log("jest event", event);
    console.log(click_to_coords(
        PLAYER_COLOR,
        event.clientX - canvasHTML.getBoundingClientRect().left,
        event.clientY - canvasHTML.getBoundingClientRect().top))
});

draw_board(PLAYER_COLOR);

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

function createGameButton() {

}

function joinGameButton() {

}