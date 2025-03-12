document.addEventListener('DOMContentLoaded', function () {
    function applyTheme(themeName) {
        document.documentElement.setAttribute('data-theme', themeName);
    }
    const savedTheme = localStorage.getItem('selectedTheme');
    if (savedTheme) {
        applyTheme(savedTheme);
    }
});
