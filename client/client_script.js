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

let lobbyHTML = document.getElementById("lobby");
let roomsHTML = document.getElementById("rooms");
let gameHTML = document.getElementById("game");
let gameIdHtml = document.getElementById("game_id");
let gameStartedHTML = document.getElementById("waiting");
let postGameHTML = document.getElementById("post_game");
let winnerTextHTML = document.getElementById("winner_text");
let rematchHTML = document.getElementById("rematchDiv");
let rematchTextHtml = document.getElementById("rematch_text");
let nameFieldHTML = document.getElementById("name_field");
let disconnectHTML = document.getElementById("opponent_disconnected");
let playerOnlineHTML = document.getElementById("player_online");
let capturedPiecesUpHTML = document.getElementById("pieces_lost_up");
let capturedPiecesDownHTML = document.getElementById("pieces_lost_down");

let in_lobby = true;
let rooms = [];
let myRoom = 0;

let playerColor = "";
let square_clicked = [];
let possible_moves = [];
let last_move = []
let is_game_over = false;
let game_started = false;
let rematch_sent = false;

let empty_board = [
    "RNBQKBNR",
    "PPPPPPPP",
    "        ",
    "        ",
    "        ",
    "        ",
    "pppppppp",
    "rnbqkbnr"
]
let board_index = 0;
let board_history = [];

let icons = {
    "p": "♟",
    "r": "♜",
    "b": "♝",
    "n": "♞",
    "q": "♛",
    "k": "♚",
    " ": " "
};

let lightColor = '#ddb180';
let lastLight = '#bfd04e';
// let darkColor = '#7c330c';
let darkColor = '#8b5c43';
let lastDark = '#7d8a28';

function draw() {
    if (!in_lobby) {
        if (game_started) {
            gameStartedHTML.style.display = "none";
        }
        lobbyHTML.style.display = "none";
        gameHTML.style.display = "block";
        if (is_game_over) {
            postGameHTML.style.display = "block";
            if (rematch_sent) {
                rematchHTML.style.display = "block";
            }
            else {
                rematchHTML.style.display = "none";
            }
        }
        else {
            postGameHTML.style.display = "none";
        }
        draw_board();
        display_captured_pieces();
    }
    else {
        lobbyHTML.style.display = "block";
        gameHTML.style.display = "none";
        while(roomsHTML.firstChild) {
            roomsHTML.removeChild(roomsHTML.firstChild);
        }

        rooms.forEach(one_room => {
            let trElement = document.createElement("tr");
            let td1 = document.createElement("td");
            // td1.textContent = "Room id: " + room;
            td1.textContent = one_room[1];
            let td2 = document.createElement("td");
            let button = document.createElement("button");
            button.onclick = () => joinGameButton(one_room[0]);
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
    let color = playerColor;

    let rowIndices = color === "white" ? [7, 6, 5, 4, 3, 2, 1, 0] : [0, 1, 2, 3, 4, 5, 6, 7];
    let colIndices = color === "white" ? [0, 1, 2, 3, 4, 5, 6, 7] : [7, 6, 5, 4, 3, 2, 1, 0];

    for (let row_on_screen = 0; row_on_screen < 8; row_on_screen++) {
        let row = rowIndices[row_on_screen];
        for (let col_on_screen = 0; col_on_screen < 8; col_on_screen++) {
            let col = colIndices[col_on_screen];
            context.fillStyle = row % 2 !== col % 2 ? lightColor : darkColor;
            if (board_index === board_history.length - 1 && last_move.length > 0 && (row === last_move[0][0] && col === last_move[0][1] || row === last_move[1][0] && col === last_move[1][1])) {
                if (context.fillStyle === darkColor) {
                    context.fillStyle = lastDark;
                } else {
                    context.fillStyle = lastLight;
                }
            }
            context.fillRect(col_on_screen * SQUARE_WIDTH, row_on_screen * SQUARE_HEIGHT, SQUARE_WIDTH, SQUARE_HEIGHT);
            context.font = "90px Arial";
            let current_board = board_history.length > 0 ? board_history[board_index] : empty_board;
            let element = current_board[row][col];
            let piece = icons[element.toLowerCase()];
            context.fillStyle = "prnbqk".includes(element) ? "black" : "white";
            context.fillText(piece, (col_on_screen + 0.05) * SQUARE_WIDTH, (row_on_screen + 0.85) * SQUARE_HEIGHT);
            if (board_index === board_history.length - 1 && possible_moves.some(x => x[0] === row && x[1] === col)) {
                draw_attacked_field(row_on_screen, col_on_screen);
            }
            draw_onboard_coordinates(row, col, row_on_screen, col_on_screen);
        }
    }
    draw_history_move_overlay();
}

function draw_history_move_overlay() {
    if (board_index < board_history.length - 1) {
        context.fillStyle = "rgba(255, 255, 255, 0.5)"
        context.fillRect(0, 0, context.canvas.width, context.canvas.height);
    }
}

function draw_onboard_coordinates(row, col, row_on_screen, col_on_screen) {
    context.font = "12px Arial";
    context.fillStyle = row_on_screen % 2 === col_on_screen % 2 ? darkColor : lightColor;
    let row_symbol = "12345678"[row];
    let col_symbol = "abcdefgh"[col];
    if (row_on_screen === 7) {
        context.fillText(col_symbol, col_on_screen * SQUARE_WIDTH + (SQUARE_WIDTH * 0.90), row_on_screen * SQUARE_HEIGHT + (SQUARE_HEIGHT * 0.95));
    }
    if (col_on_screen === 0) {
        context.fillText(row_symbol, col_on_screen * SQUARE_WIDTH + (SQUARE_WIDTH * 0.03), row_on_screen * SQUARE_HEIGHT + (SQUARE_HEIGHT * 0.13));
    }
}

function draw_attacked_field(row_on_screen, col_on_screen) {
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

function display_captured_pieces() {
    let pieces = "prnbqk";
    let counter = {}
    for (const p of pieces) {
        let count = 2;
        if (p === 'p'){
            count = 8;
        }
        else if (p === 'q' || p === 'k') {
            count = 1;
        }
        counter[p] = count;
        counter[p.toUpperCase()] = count;
    }
    for (let row = 0; row < 8; row++) {
        for (let col = 0; col < 8; col++) {
            let current_board = board_history.length > 0 ? board_history[board_index] : empty_board;
            let elem = current_board[row][col];
            if (elem in counter) {
                let new_val = counter[elem] - 1;
                counter[elem] = new_val >= 0 ? new_val : 0;
            }
        }
    }
    let transform = (arr) => arr.split('').map(x => x.repeat(counter[x])).join('');
    let captured_by_black = transform("PNBRQ");
    let captured_by_white = transform("pnbrq");
    let to_str = (arr) => arr.split('').map(x => icons[x.toLowerCase()]).join('');
    let [white_text, black_text] = [to_str(captured_by_white), to_str(captured_by_black)];
    let outline_style = "0 0 2px black, 0 0 2px black, 0 0 2px black, 0 0 2px black";
    if (playerColor === "white") {
        capturedPiecesUpHTML.style.textShadow = outline_style;
        capturedPiecesUpHTML.style.color = "white";
        capturedPiecesUpHTML.textContent = black_text;
        capturedPiecesDownHTML.style.color = "black";
        capturedPiecesDownHTML.textContent = white_text;
    }
    else {
        capturedPiecesUpHTML.style.color = "black";
        capturedPiecesUpHTML.textContent = white_text;
        capturedPiecesDownHTML.style.textShadow = outline_style;
        capturedPiecesDownHTML.style.color = "white";
        capturedPiecesDownHTML.textContent = black_text;
    }
}

function createGameButton() {
    let msg = {"msg_type": "Create", "room_id": 0, "room_name": nameFieldHTML.value};
    send_socket(msg);
}

function joinGameButton(room) {
    let msg = {"msg_type": "Join", "room_id": room};
    send_socket(msg);
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
        send_socket(msg);
        cancel_move();
    }
}

function start_move(coords) {
    if (!is_game_over) {
        let msg = {"msg_type": "Possible", "room_id": myRoom, "possible_moves": coords};
        send_socket(msg);
    }
}

function cancel_move() {
    possible_moves = [];
}

function is_move_possible(coords) {
    return possible_moves.some(a => compare_arrays(a, coords));
}

function rematch_offer() {
    let msg = {"msg_type": "Rematch", "room_id": myRoom};
    send_socket(msg);
}

function send_socket(msg) {
    console.log("Sending message:");
    let json_msg = JSON.stringify(msg);
    console.log(json_msg);
    console.log(msg);
    socket.send(json_msg);
}

function reset_game() {
    playerColor = "";
    square_clicked = [];
    possible_moves = [];
    last_move = []
    is_game_over = false;
    game_started = false;
    rematch_sent = false;
    board_history = [];
    board_index = -1;
}

function set_room_name() {
    if (nameFieldHTML.value !== "") {
        localStorage.setItem("room_name", nameFieldHTML.value);
    }
    else {
        let name = random_name();
        localStorage.setItem("room_name", name);
        nameFieldHTML.value = name;
    }
}

function random_name() {
    let part1 = ["Good", "Bad", "Nice", "Cool", "Big", "Small", "Huge", "Shy",
        "Red", "Blue", "Green", "Yellow", "Orange", "Purple", "Black", "White"];
    let part2 = ["Dog", "Bird", "Snake", "Snail", "Parrot", "Fox", "Bug", "Fly", "Pig", "Horse", "Cow",
        "Box", "Car", "Rat", "Apple", "Ball", "Tree"];

    return pick(part1) + pick(part2) + Math.floor(Math.random() * 1000) + "'s room";
}

function pick(arr) {
    return arr[Math.floor(Math.random() * arr.length)];
}

function exit_action() {
    console.log("exit");
    socket.close();
    location.reload();
}

function navigation_left() {
    board_index = board_index - 1 >= 0 ? board_index - 1 : board_index;
    draw();
    // if draw_board, captured pieced are not revered in time, which is better?
    // draw_board();
}

function navigation_right() {
    board_index = board_index + 1 < board_history.length ? board_index + 1 : board_index;
    draw();
}

canvasHTML.addEventListener("mousedown", event => {
    if (board_index < board_history.length - 1) {
        board_index = board_history.length - 1;
    }
    else if(game_started) {
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

document.onkeydown = e => {
    if (e.key === "ArrowLeft") {
        navigation_left();
    }
    else if (e.key === "ArrowRight") {
        navigation_right();
    }
};

socket.addEventListener("message", event => {
    console.log("message received:", event.data);
    let decoded = JSON.parse(event.data);
    console.log("message decoded", decoded);
    if (decoded === "Disconnected") {
        socket.close();
        disconnectHTML.style.display = "block";
    }
    else if (decoded["msg_type"] === "NewRoom") {
        reset_game();
        gameIdHtml.textContent = nameFieldHTML.value;
        myRoom = decoded["room_id"];
        playerColor = decoded["color"];
        in_lobby = false;
    }
    else if (decoded["msg_type"] === "Possible") {
        possible_moves = decoded["possible_moves"];
    }
    else if (decoded["msg_type"] === "GameResultWhiteWon") {
        winnerTextHTML.textContent = "Game over, white won!";
        is_game_over = true;
    }
    else if (decoded["msg_type"] === "GameResultBlackWon") {
        winnerTextHTML.textContent = "Game over, black won!";
        is_game_over = true;
    }
    else if (decoded["msg_type"] === "GameResultDraw") {
        winnerTextHTML.textContent = "Game over, draw!";
        is_game_over = true;
    }
    else if ("Rematch" in decoded) {
        rematch_sent = true;
        if (decoded["Rematch"]["my_offer"]) {
            rematchTextHtml.textContent = "Rematch offer sent, waiting for the opponent...";
        }
        else {
            rematchTextHtml.textContent = "Your opponent offers a rematch...";
        }
    }
    else if ("Board" in decoded) {
        game_started = true;
        board_index = board_history.length;
        board_history.push(parse_board(decoded["Board"]["current_board"]));
        if (decoded["Board"]["last_move"] !== null) {
            last_move = decoded["Board"]["last_move"];
        }
        cancel_move();
    }
    else if ("Rooms" in decoded) {
        rooms = decoded["Rooms"]["room_names"];
    }
    else if ("PlayersOnline" in decoded) {
        playerOnlineHTML.textContent = "Players online: " + decoded["PlayersOnline"]["count"];
    }

    draw();
});

setInterval(() => {
    let msg = {"msg_type": "Ping", "room_id": myRoom};
    send_socket(msg);
}, 59_000);

if (!("room_name" in localStorage)) {
    localStorage.setItem("room_name", random_name());
}
nameFieldHTML.value = localStorage.getItem("room_name");

reset_game();
draw();
