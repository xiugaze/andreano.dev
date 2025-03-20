const host = "http://localhost:8080"
const get_endpoint = "/comments"
const post_endpoint = "/comments"

const POST_NAME = document.body.dataset.post_id || "test"

const options = {
  year: 'numeric',
  month: '2-digit',
  day: '2-digit',
  hour: '2-digit',
  minute: '2-digit',
  second: '2-digit',
  hour12: false
};

window.onload = function() {
    fetch(`${host}${get_endpoint}?post=${POST_NAME}`)
        .then(response => response.json())
        .then(comments => {
            const container = document.getElementById('comments');
            container.innerHTML = ''; 
            comments.forEach(comment => {
                const div = document.createElement('div');
                div.className = 'comment';
                div.innerHTML = `
                    <strong>${comment.author}</strong> 
                    <small>${new Date(comment.timestamp).toLocaleString('en-CA', options)
                        .replace(/(\d{4})-(\d{2})-(\d{2}), (\d{2}):(\d{2}):(\d{2})/, '<$1-$2-$3> $4:$5:$6')}</small>
                    <p>${comment.content}</p>
                `;
                container.appendChild(div);
            });
        })
        .catch(error => console.error('Error fetching comments:', error));
};


document.getElementById('challengeButton').onclick = function(e) {
    e.preventDefault();
    fetch(`${host}/challenge`)
        .then(response => response.json())
        .then(challenge => JSON.parse(challenge))
        .then(challenge => {
            document.getElementById("commentSubmit").hidden = false;
            document.getElementById("challengeButton").hidden = true;
            window.alert(`Challenge: ($CURRENT_YEAR**${challenge.p}) mod ${challenge.q}`);
            document.getElementById('challenge_id').value = challenge.id;
        });
}

document.getElementById('commentForm').onsubmit = function(e) {
    e.preventDefault();
    const author = this.author.value;
    const content = this.content.value;
    const id = this.challenge_id.value;
    const sum = parseInt(this.challenge_response.value);

    const body = JSON.stringify({ id, sum, post: POST_NAME, author, content });
    console.log(body);
    
    fetch(`${host}${post_endpoint}`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: body
    })
    .then(response => {
        if (!response.ok) throw new Error('Failed to post comment');
        return response.json();
    })
    .then(comment => {
        const container = document.getElementById('comments');
        const div = document.createElement('div');
        div.className = 'comment';
        div.innerHTML = `
            <strong>${comment.author}</strong> 
            <small>${new Date(comment.timestamp).toLocaleString('en-CA', options)
                        .replace(/(\d{4})-(\d{2})-(\d{2}), (\d{2}):(\d{2}):(\d{2})/, '<$1-$2-$3> $4:$5:$6')}</small>
            <p>${comment.content}</p>
        `;
        container.appendChild(div);
        this.reset();
    })
    .catch(error => {
            console.error('Error posting comment:', error);
            window.alert("Error posting comment (is the challenge solution correct?");
    });
};
