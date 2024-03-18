console.log("script start!");

let canvasHTML = document.getElementById("chess-board");
let context = canvasHTML.getContext("2d");
let SQUARE_WIDTH = context.canvas.width / 8;
let SQUARE_HEIGHT = context.canvas.height / 8;
console.log(SQUARE_WIDTH, SQUARE_HEIGHT);

let in_lobby = true;
let game_started = false;

let lobbyHTML = document.getElementById("lobby");
let roomsHTML = document.getElementById("rooms");
let gameHTML = document.getElementById("game");
let gameIdHtml = document.getElementById("game_id");
let gameStartedHTML = document.getElementById("waiting");

let rooms = [];
let myRoom = 0;
let myColor = "";
let square_clicked = [];

let current_board = [
    "        ",
    "        ",
    "        ",
    "        ",
    "        ",
    "        ",
    "        ",
    "        "
]

let icons = {
    "p": "♟︎",
    "r": "♜",
    "b": "♝",
    "n": "♞",
    "q": "♛",
    "k": "♚",
    " ": " "
};

function draw() {
    if (!in_lobby) {
        if (game_started) {
            gameStartedHTML.style.display = "none";
        }
        lobbyHTML.style.display = "none";
        gameHTML.style.display = "block";
        draw_board();
    }
    else {
        lobbyHTML.style.display = "block";
        gameHTML.style.display = "none";
        while(roomsHTML.firstChild) {
            roomsHTML.removeChild(roomsHTML.firstChild);
        }

        rooms.forEach(room => {
            let trElement = document.createElement("tr");
            let td1 = document.createElement("td");
            td1.textContent = "Room id: " + room;
            let td2 = document.createElement("td");
            let button = document.createElement("button");
            button.onclick = () => joinGameButton(room);
            button.textContent = "Join";

            trElement.appendChild(td1);
            td2.appendChild(button);
            trElement.appendChild(td2);
            roomsHTML.appendChild(trElement);
        });
    }
}

function parse_board(boardStr) {
    return boardStr.split("\n");
}

function click_to_coords(color, x, y) {
    let row = Math.floor(y / SQUARE_HEIGHT);
    let col = Math.floor(x / SQUARE_WIDTH);
    if(color === "white") {
        row = Math.floor((context.canvas.height - y) / SQUARE_HEIGHT);
    }
    if(color === "black") {
        col = Math.floor((context.canvas.width - x) / SQUARE_WIDTH);
    }
    row = Math.min(7, row);
    row = Math.max(0, row);
    col = Math.min(7, col);
    col = Math.max(0, col);
    return [row, col];
}

function draw_board() {
    let lightColor = '#ddb180';
    let darkColor = '#7c330c';
    let color = myColor;

    // let board = parse_board(color, "RNBQKBNR\nPPPPPPPP\n        \n        \n        \n        \npppppppp\nrnbqkbnr");

    let rowIndices = color == "white" ? [7, 6, 5, 4, 3, 2, 1, 0] : [0, 1, 2, 3, 4, 5, 6, 7];
    let colIndices = color == "white" ? [0, 1, 2, 3, 4, 5, 6, 7] : [7, 6, 5, 4, 3, 2, 1, 0];

    for (let row_on_screen= 0; row_on_screen < 8; row_on_screen++) {
        let row = rowIndices[row_on_screen];
        for (let col_on_screen = 0; col_on_screen < 8; col_on_screen++) {
            let col = colIndices[col_on_screen];
            context.fillStyle = row % 2 != col % 2 ? lightColor : darkColor;
            context.fillRect(col_on_screen * SQUARE_WIDTH, row_on_screen * SQUARE_HEIGHT, SQUARE_WIDTH, SQUARE_HEIGHT);
            context.font = "90px Arial";
            let element = current_board[row][col];
            let piece = icons[element.toLowerCase()];
            context.fillStyle = "prnbqk".includes(element) ? "black" : "white";
            context.fillText(piece, (col_on_screen + 0.05) * SQUARE_WIDTH, (row_on_screen + 0.85) * SQUARE_HEIGHT);
        }
    }
}

canvasHTML.addEventListener("mousedown", event => {
    if(game_started) {
        square_clicked = click_to_coords(
            myColor,
            event.clientX - canvasHTML.getBoundingClientRect().left,
            event.clientY - canvasHTML.getBoundingClientRect().top);
    }
});

canvasHTML.addEventListener("mouseup", event => {
    if(game_started) {
        console.log("jest event", event);
        let coords = click_to_coords(
            myColor,
            event.clientX - canvasHTML.getBoundingClientRect().left,
            event.clientY - canvasHTML.getBoundingClientRect().top);
        console.log(coords);

        if (square_clicked[0] !== coords[0] || square_clicked[1] !== coords[1]) {
            let msg = {"msg_type": "Move", "make_move": [square_clicked, coords], "room_id": myRoom};
            console.log("Sending: ", msg);
            socket.send(JSON.stringify(msg));
        }
    }
});

// const socket = new WebSocket("ws://127.0.0.1:9977");
const socket = new WebSocket("ws://4.223.103.5:9977");
// socket.addEventListener("open", (event) => {
//     console.log("socket open event: ", event);
//     let m = {"msg_type": "Join", "room_id": 123, "make_move": [[1,1], [2,2]]}
//     socket.send(JSON.stringify(m));
//     console.log("message sent");
// });

socket.addEventListener("message", event => {
    console.log("message received:", event.data);
    let decoded = JSON.parse(event.data);
    if(decoded["msg_type"] === "Rooms") {
        rooms = decoded["rooms"];
    }
    else if(decoded["msg_type"] === "NewRoom") {
        gameIdHtml.textContent = "Room ID: " + decoded["room_id"];
        myRoom = decoded["room_id"];
        myColor = decoded["color"];
        in_lobby = false;
    }
    else if(decoded["msg_type"] === "Board") {
        game_started = true;
        current_board = parse_board(decoded["board"]);
    }

    draw();
});

function createGameButton() {
    let msg = {"msg_type": "Create"};
    console.log("sending", msg);
    socket.send(JSON.stringify(msg));
}

function joinGameButton(roomId) {
    let msg = {"msg_type": "Join", "room_id": roomId};
    console.log("sending", msg);
    socket.send(JSON.stringify(msg));
}

draw();