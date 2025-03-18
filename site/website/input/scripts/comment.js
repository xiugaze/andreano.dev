const host = "http://localhost:8080"
const get_endpoint = "/get-comments"
const post_endpoint = "/comments"

const POST_NAME = "test"; // Hardcoded for this example; could be dynamic

// Load comments on page load for specific post
window.onload = function() {
    fetch(`${host}${get_endpoint}?post=${POST_NAME}`)
        .then(response => response.json())
        .then(comments => {
            const container = document.getElementById('comments');
            container.innerHTML = ''; // Clear existing comments
            comments.forEach(comment => {
                const div = document.createElement('div');
                div.className = 'comment';
                div.innerHTML = `
                    <strong>${comment.author}</strong> 
                    <small>${new Date(comment.timestamp).toLocaleString()}</small>
                    <p>${comment.content}</p>
                `;
                container.appendChild(div);
            });
        })
        .catch(error => console.error('Error fetching comments:', error));
};

// Handle form submission with JSON for specific post
document.getElementById('commentForm').onsubmit = function(e) {
    e.preventDefault();
    const author = this.author.value;
    const content = this.content.value;
    
    fetch(`${host}${post_endpoint}`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({ post: POST_NAME, author, content })
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
            <small>${new Date(comment.timestamp).toLocaleString()}</small>
            <p>${comment.content}</p>
        `;
        container.insertBefore(div, container.firstChild);
        this.reset();
    })
    .catch(error => console.error('Error posting comment:', error));
};
