function update(app, model) {
    let t = app.elapsedSeconds()
    model.radius = model.radius + t
    return model
}