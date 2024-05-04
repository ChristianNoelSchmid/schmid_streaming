const video = document.querySelector('video');
const nextUrl = document.querySelector('#next-url').textContent;

video.addEventListener('ended', () => {
    if(nextUrl !== '') {
        window.location = nextUrl;
    }
});