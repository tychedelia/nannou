function update(app, model) {
    let t = app.time()
    model.radius = t * 10.0;
    return model
}
