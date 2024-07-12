function update(app, model) {
    let t = app.elapsedSeconds()
    model.set('radius', model.get('radius') + t)

    return model
}