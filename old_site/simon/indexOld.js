
let buttonColors = ["red", "blue", "green", "yellow"];

let gamePattern = [];
// stack of correct inputs
let userClickedPattern = [];       
// stack of user inputs
let level = 0;

let turn = false;

// call main function
$(document).on("keydown", function() {
    $(document).off();
    start();
});


function nextSequence() {
    ++level;
    $("h1").html("Level " + level);
    let rand = Math.floor(Math.random() * 4);
    let chosenColor = buttonColors[rand];
    gamePattern.push(chosenColor);
    gamePattern.forEach(e => {
        playSound(e);
        $("#" + e).fadeOut(100).fadeIn(100);
    });
    
    console.log("game: " + gamePattern);
    turn = true;
}

function playSound(name) {
    let audio = new Audio("sounds/" + name + ".mp3");
    audio.play();
}

function animatePress(currentColor) {
    $("#" + currentColor).addClass("pressed");
    setTimeout(function () {
        $("#" + currentColor).removeClass("pressed");
    }, 100);
}

function checkAnswer(currentLevel) {
    if(userClickedPattern[currentLevel] === gamePattern[currentLevel]) {
        console.log("success");
    } else {
        console.log("wrong");
    }
}

function start() {
    lost = false;

    $(".btn").on("click", function(e) {
        let userColor = e.target.id;
        playSound(userColor);
        animatePress(userColor);
        userClickedPattern.push(userColor);
        console.log("player: " + userClickedPattern);
        checkAnswer(level);
        turn = false;
    });

    while(!lost) {
        nextSequence();
        while(turn = true) {
            
        }
    }
    


    // $("#" + chosenColor).fadeOut(85).fadeIn(75);

}


