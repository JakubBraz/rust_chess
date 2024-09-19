// const socket = new WebSocket("ws://127.0.0.1:9977");
const socket = new WebSocket("ws://4.223.103.5:9977");

let canvasHTML = document.getElementById("chess-board");
let context = canvasHTML.getContext("2d");
let SQUARE_WIDTH = context.canvas.width / 8;
let SQUARE_HEIGHT = context.canvas.height / 8;
console.log(SQUARE_WIDTH, SQUARE_HEIGHT);
let LINE_SIZE = SQUARE_WIDTH * 0.075;
let LINE_LEN = SQUARE_HEIGHT * 0.2;
// let LINE_OFFSET = LINE_SIZE * 5;
let LINE_OFFSET = LINE_SIZE;

let in_lobby = true;
let game_started = false;

let lobbyHTML = document.getElementById("lobby");
let roomsHTML = document.getElementById("rooms");
let gameHTML = document.getElementById("game");
let gameIdHtml = document.getElementById("game_id");
let gameStartedHTML = document.getElementById("waiting");
let postGameHTML = document.getElementById("post_game");
let winnerTextHTML = document.getElementById("winner_text");

let rooms = [];
let myRoom = 0;
let playerColor = "";
let square_clicked = [];
let possible_moves = [];
let last_move = []

let is_game_over = false;

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
        if (is_game_over) {
            postGameHTML.style.display = "block";
        }
        else {
            postGameHTML.style.display = "none";
        }
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
    let lastLight = '#bfd04e';
    // let darkColor = '#7c330c';
    let darkColor = '#8b5c43';
    let lastDark = '#7d8a28';

    let color = playerColor;

    let rowIndices = color === "white" ? [7, 6, 5, 4, 3, 2, 1, 0] : [0, 1, 2, 3, 4, 5, 6, 7];
    let colIndices = color === "white" ? [0, 1, 2, 3, 4, 5, 6, 7] : [7, 6, 5, 4, 3, 2, 1, 0];

    for (let row_on_screen = 0; row_on_screen < 8; row_on_screen++) {
        let row = rowIndices[row_on_screen];
        for (let col_on_screen = 0; col_on_screen < 8; col_on_screen++) {
            let col = colIndices[col_on_screen];
            context.fillStyle = row % 2 !== col % 2 ? lightColor : darkColor;
            if (last_move.length > 0 && (row === last_move[0][0] && col === last_move[0][1] || row === last_move[1][0] && col === last_move[1][1])) {
                if (context.fillStyle === darkColor) {
                    context.fillStyle = lastDark;
                }
                else {
                    context.fillStyle = lastLight;
                }
            }
            context.fillRect(col_on_screen * SQUARE_WIDTH, row_on_screen * SQUARE_HEIGHT, SQUARE_WIDTH, SQUARE_HEIGHT);
            context.font = "90px Arial";
            let element = current_board[row][col];
            let piece = icons[element.toLowerCase()];
            context.fillStyle = "prnbqk".includes(element) ? "black" : "white";
            context.fillText(piece, (col_on_screen + 0.05) * SQUARE_WIDTH, (row_on_screen + 0.85) * SQUARE_HEIGHT);
            if (possible_moves.some(x => x[0] === row && x[1] === col)) {
                draw_attacked_field(row_on_screen, col_on_screen);
            }
        }
    }
}

function draw_attacked_field(row_on_screen, col_on_screen) {
    let line_size = 0.3;
    context.fillStyle = "#2fae01";
    context.fillRect(col_on_screen * SQUARE_WIDTH + LINE_OFFSET, row_on_screen * SQUARE_HEIGHT + LINE_OFFSET, LINE_SIZE, LINE_LEN);
    context.fillRect(col_on_screen * SQUARE_WIDTH + LINE_OFFSET, row_on_screen * SQUARE_HEIGHT + LINE_OFFSET, LINE_LEN, LINE_SIZE);

    context.fillRect(col_on_screen * SQUARE_WIDTH + SQUARE_WIDTH - LINE_SIZE - LINE_OFFSET, row_on_screen * SQUARE_HEIGHT + LINE_OFFSET, LINE_SIZE, LINE_LEN);
    context.fillRect(col_on_screen * SQUARE_WIDTH + SQUARE_WIDTH - LINE_LEN - LINE_OFFSET, row_on_screen * SQUARE_HEIGHT + LINE_OFFSET, LINE_LEN, LINE_SIZE);

    context.fillRect(col_on_screen * SQUARE_WIDTH + LINE_OFFSET, row_on_screen * SQUARE_HEIGHT + SQUARE_HEIGHT - LINE_LEN - LINE_OFFSET, LINE_SIZE, LINE_LEN);
    context.fillRect(col_on_screen * SQUARE_WIDTH + LINE_OFFSET, row_on_screen * SQUARE_HEIGHT + SQUARE_HEIGHT - LINE_SIZE - LINE_OFFSET, LINE_LEN, LINE_SIZE);

    context.fillRect(col_on_screen * SQUARE_WIDTH + SQUARE_WIDTH - LINE_SIZE - LINE_OFFSET, row_on_screen * SQUARE_HEIGHT + SQUARE_HEIGHT - LINE_LEN - LINE_OFFSET, LINE_SIZE, LINE_LEN);
    context.fillRect(col_on_screen * SQUARE_WIDTH + SQUARE_WIDTH - LINE_LEN - LINE_OFFSET, row_on_screen * SQUARE_HEIGHT + SQUARE_HEIGHT - LINE_SIZE - LINE_OFFSET, LINE_LEN, LINE_SIZE);
}

function createGameButton() {
    let msg = {"msg_type": "Create", "room_id": 0};
    console.log("sending", msg);
    socket.send(JSON.stringify(msg));
}

function joinGameButton(roomId) {
    let msg = {"msg_type": "Join", "room_id": roomId};
    console.log("sending", msg);
    socket.send(JSON.stringify(msg));
}

function compare_arrays(a, b) {
    if (a.length !== b.length) {
        return false;
    }
    for (let i = 0; i < a.length; i++) {
        if (a[i] !== b[i]) {
            return false;
        }
    }
    return true;
}

function make_move(first_click, second_click) {
    if (first_click.length === 2 && second_click.length === 2) {
        let msg = {"msg_type": "Move", "make_move": [first_click, second_click], "room_id": myRoom};
        console.log("Sending: ", msg);
        socket.send(JSON.stringify(msg));
        cancel_move();
    }
}

function start_move(coords) {
    if (!is_game_over) {
        let msg = {"msg_type": "Possible", "room_id": myRoom, "possible_moves": coords};
        socket.send(JSON.stringify(msg));
    }
}

function cancel_move() {
    possible_moves = [];
}

function is_move_possible(coords) {
    return possible_moves.some(a => compare_arrays(a, coords));
}

canvasHTML.addEventListener("mousedown", event => {
    if(game_started) {
        let coords = click_to_coords(
            playerColor,
            event.clientX - canvasHTML.getBoundingClientRect().left,
            event.clientY - canvasHTML.getBoundingClientRect().top);
        if (is_move_possible(coords)) {
            make_move(square_clicked, coords);
        }
        else {
            start_move(coords);
        }
        square_clicked = coords;
        draw();
    }
});

canvasHTML.addEventListener("mouseup", event => {
    if(game_started) {
        let coords = click_to_coords(
            playerColor,
            event.clientX - canvasHTML.getBoundingClientRect().left,
            event.clientY - canvasHTML.getBoundingClientRect().top);

        if (!compare_arrays(coords, square_clicked)) {
            make_move(square_clicked, coords);
        }
        draw();
    }
});

socket.addEventListener("message", event => {
    console.log("message received:", event.data);
    let decoded = JSON.parse(event.data);
    console.log("message decoded", decoded);
    if(decoded["msg_type"] === "Rooms") {
        rooms = decoded["rooms"];
    }
    else if(decoded["msg_type"] === "NewRoom") {
        gameIdHtml.textContent = "Room ID: " + decoded["room_id"];
        myRoom = decoded["room_id"];
        playerColor = decoded["color"];
        in_lobby = false;
    }
    else if("Board" in decoded) {
        game_started = true;
        current_board = parse_board(decoded["Board"]["current_board"]);
        if (decoded["Board"]["last_move"] !== null) {
            last_move = decoded["Board"]["last_move"];
        }
        cancel_move();
    }
    else if(decoded["msg_type"] === "Possible") {
        possible_moves = decoded["possible_moves"];
    }
    else if(decoded["msg_type"] === "GameResultWhiteWon") {
        winnerTextHTML.textContent = "Game over, white won!";
        is_game_over = true;
    }
    else if(decoded["msg_type"] === "GameResultBlackWon") {
        winnerTextHTML.textContent = "Game over, black won!";
        is_game_over = true;
    }
    else if(decoded["msg_type"] === "GameResultDraw") {
        winnerTextHTML.textContent = "Game over, draw!";
        is_game_over = true;
    }

    draw();
});

draw();
