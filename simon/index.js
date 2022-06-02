let buttonColors = ["red", "blue", "green", "yellow"];
let userClickedPattern = [];
let gamePattern = [];
let started = false;
let level = 0;

$(document).keydown(function() {
  if (!started) {
    $("#level-title").text("Level " + level);
    nextSequence();
    started = true;
  }
});

function buttonOnClick() {
    let userChosenColor = $(this).attr("id");
    userClickedPattern.push(userChosenColor);
    playSound(userChosenColor);
    animatePress(userChosenColor);
    checkAnswer(userClickedPattern.length - 1);
}

function playRestart() {
    $(".playbtn").addClass("pressed");
    setTimeout(function () {
        $(".playbtn").removeClass("pressed");
    }, 100);
    if(started) {
        startOver();

        $("body").addClass("restart");
        $("#level-title").text("Restarting...");

        setTimeout(function () {
            $("body").removeClass("restart");
            started = true;
            nextSequence();
        }, 200);

    } else {
        started = true;
        nextSequence();
        $("#level-title").text("Level " + level);
    }
}

$(".playbtn").on("click", playRestart);

function nextSequence() {
    $(".btn").off();
    $(".play-restart").removeClass("fa-play");
    $(".play-restart").addClass("fa-arrow-rotate-left");

    level++;
    $("#level-title").text("Level " + level);

    let randomNumber = Math.floor(Math.random() * 4);
    let randomChosenColor = buttonColors[randomNumber];
    gamePattern.push(randomChosenColor);

    $("#" + randomChosenColor).fadeIn(100).fadeOut(100).fadeIn(100);
    playSound(randomChosenColor);
    $(".btn").on("click", buttonOnClick);
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
        if(currentLevel === gamePattern.length - 1) {
            setTimeout(nextSequence, 1000);
            userClickedPattern = [];
        }
    } else {
        playSound("wrong");
        $("body").addClass("game-over");
        setTimeout(function () {
            $("body").removeClass("game-over");
        }, 200);
        $("#level-title").text("Game Over, Press Any Key to Restart")
        startOver();
    }
}

function startOver() {
    level = 0;
    gamePattern = [];
    userClickedPattern = [];
    started = false;
    $(".btn").off();
}


