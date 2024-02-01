#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>
#include "emulatorthread.h"

QT_BEGIN_NAMESPACE
namespace Ui {
class MainWindow;
}
QT_END_NAMESPACE

class MainWindow : public QMainWindow
{
    Q_OBJECT

public:
    MainWindow(QWidget* parent = nullptr);
    ~MainWindow();

public slots:
    void onFrameGenerated(QPixmap frame);
    void onRegistersFetched(Registers registers);

private slots:
    void on_btn_pause_clicked();

signals:
    void pause();

private:
    Ui::MainWindow* ui;
    EmulatorThread* thread;

    void log(const char* msg, ...);
};
#endif // MAINWINDOW_H
