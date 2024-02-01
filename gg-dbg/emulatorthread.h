#ifndef EMULATORTHREAD_H
#define EMULATORTHREAD_H

#include <QThread>
#include <QPixmap>
#include "gg.hpp"

class EmulatorThread : public QThread
{
    Q_OBJECT

signals:
    void frameGenerated(QPixmap frame);
    void registersFetched(Registers registers);

public slots:
    void onPause();

public:
    void run();

private:
    const size_t INTERNAL_WIDTH = 256;
    const size_t INTERNAL_HEIGHT = 224;
    bool paused = false;
};

#endif // EMULATORTHREAD_H
