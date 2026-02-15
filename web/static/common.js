btn_mode_immediate.onclick = () => {
    window.location.href = '/';
};

btn_mode_template.onclick = () => {
    window.location.href = '/template';
};

btn_mode_solid_color.onclick = () => {
    window.location.href = '/solid-color';
};

async function postImage(endpoint, imageData) {
    return await fetch(endpoint, {
        method: 'POST',
        headers: new Headers({
            'Content-Type': 'application/octet-stream'
        }),
        body: imageData
    });
}

function setImageUrl(img, imageBytes) {
    URL.revokeObjectURL(img.src);

    const blob = new Blob([imageBytes], { type: 'application/octet-binary' });
    const objUrl = URL.createObjectURL(blob);

    img.src = objUrl;
}
