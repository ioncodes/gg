#ifndef EMULATORTHREAD_H
#define EMULATORTHREAD_H

#include <QThread>
#include <QPixmap>

class EmulatorThread : public QThread
{
    Q_OBJECT

signals:
    void frameGenerated(QPixmap frame);

public:
    void run();

private:
    const size_t INTERNAL_WIDTH = 256;
    const size_t INTERNAL_HEIGHT = 224;
};

#endif // EMULATORTHREAD_H
