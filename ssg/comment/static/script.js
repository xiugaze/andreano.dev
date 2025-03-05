// Base URL for the comment engine
const COMMENT_API = 'http://localhost:4000';

// Fetch and display comments for all posts
async function loadComments() {
    try {
        const response = await fetch(`${COMMENT_API}/fetch`);
        const comments = await response.json();
        
        // Group comments by postId
        const commentsByPost = {};
        comments.forEach(comment => {
            if (comment.postId) {
                if (!commentsByPost[comment.postId]) {
                    commentsByPost[comment.postId] = [];
                }
                commentsByPost[comment.postId].push(comment);
            }
        });

        // Display comments for each post
        document.querySelectorAll('.comments').forEach(commentDiv => {
            const postId = commentDiv.getAttribute('data-post-id');
            commentDiv.innerHTML = ''; // Clear existing comments
            const postComments = commentsByPost[postId] || [];
            postComments.forEach(comment => {
                const div = document.createElement('div');
                div.className = 'comment';
                div.innerHTML = `<strong>${comment.author}</strong>: ${comment.text}`;
                commentDiv.appendChild(div);
            });
        });
    } catch (error) {
        console.error('Failed to load comments:', error);
    }
}

// Handle comment form submissions
document.querySelectorAll('.comment-form').forEach(form => {
    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const postId = form.getAttribute('data-post-id');
        const author = form.querySelector('input[name="author"]').value;
        const text = form.querySelector('textarea[name="text"]').value;

        const comment = {
            postId,
            author,
            text,
            timestamp: new Date().toISOString()
        };

        try {
            const response = await fetch(`${COMMENT_API}/post`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(comment)
            });
            const result = await response.json();
            if (result.status === 'Comment added') {
                form.reset(); // Clear the form
                loadComments(); // Refresh comments
            } else {
                alert('Failed to add comment');
            }
        } catch (error) {
            console.error('Error posting comment:', error);
            alert('Error posting comment');
        }
    });
});

// Load comments when the page loads
document.addEventListener('DOMContentLoaded', loadComments);
