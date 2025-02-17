import PocketBase from 'pocketbase';

// Initialize PocketBase client
const pb = new PocketBase('http://127.0.0.1:8090');

interface Comment {
    id: string;
    author: string;
    content: string;
}

// Fetch comments from PocketBase
async function fetchComments(): Promise<Comment[]> {
    const records = await pb.collection('comments').getFullList();
    return records.map((record) => ({
        id: record.id,
        author: record.author,
        content: record.content,
    }));
}

// Add a new comment to PocketBase
async function addComment(author: string, content: string): Promise<void> {
    await pb.collection('comments').create({
        author,
        content,
    });
}

// Render comments to the page
function renderComments(comments: Comment[]) {
    const commentsList = document.getElementById('comments-list');
    if (commentsList) {
        commentsList.innerHTML = comments
            .map(
                (comment) => `
                <div class="comment">
                    <strong>${comment.author}</strong>
                    <p>${comment.content}</p>
                </div>
            `
            )
            .join('');
    }
}

// Handle form submission
async function handleFormSubmit(event: Event) {
    event.preventDefault();

    const authorInput = document.getElementById('author') as HTMLInputElement;
    const contentInput = document.getElementById('content') as HTMLTextAreaElement;

    await addComment(authorInput.value, contentInput.value);

    // Reload comments
    const comments = await fetchComments();
    renderComments(comments);

    // Clear the form
    authorInput.value = '';
    contentInput.value = '';
}

// Initialize the app
async function init() {
    const comments = await fetchComments();
    renderComments(comments);

    const form = document.getElementById('comment-form');
    if (form) {
        form.addEventListener('submit', handleFormSubmit);
    }
}

// Run the app
init();
