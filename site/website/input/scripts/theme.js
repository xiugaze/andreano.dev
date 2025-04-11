document.addEventListener('DOMContentLoaded', function () {
    const icons = document.querySelectorAll('.icon');
    let mainIcon;
    function applyTheme(themeName) {
        document.documentElement.setAttribute('data-theme', themeName);
    }
    function saveSelection(iconData, theme) {
        localStorage.setItem('mainIconData', iconData);
        localStorage.setItem('selectedTheme', theme);
    }
    function loadSavedPreferences() {
        const savedIconData = localStorage.getItem('mainIconData');
        const savedTheme = localStorage.getItem('selectedTheme');
        if (savedIconData) {
            const savedIcon = document.querySelector(`.icon[data-icon="${savedIconData}"]`);
            if (savedIcon) {
                icons.forEach(i => i.classList.remove('main-icon'));
                savedIcon.classList.add('main-icon');
                mainIcon = savedIcon;
                return;
            }
        }
        mainIcon = document.querySelector('.main-icon');
        if (mainIcon && !savedTheme) {
            applyTheme(mainIcon.getAttribute('data-theme'));
        }
    }
    loadSavedPreferences();

    const earlyStyleElement = document.querySelector('style[data-early-icon-fix]');
    if (earlyStyleElement) {
        earlyStyleElement.remove();
    }

    icons.forEach(icon => {
        icon.addEventListener('click', function () {
            if (this === mainIcon) return;
            icons.forEach(i => i.classList.remove('main-icon'));
            this.classList.add('main-icon');
            mainIcon = this;
            const theme = this.getAttribute('data-theme');
            applyTheme(theme);
            saveSelection(this.getAttribute('data-icon'), theme);
        });
    });
});
